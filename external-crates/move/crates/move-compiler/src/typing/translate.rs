// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    core::{self, Context, Local, Subst},
    expand, infinite_instantiations, recursive_datatypes,
};
use crate::{
    diag,
    diagnostics::{codes::*, Diagnostic},
    editions::{Edition, FeatureGate, Flavor},
    expansion::ast::{
        AbilitySet, Attribute, AttributeValue_, Attribute_, DottedUsage, Fields, Friend,
        ModuleAccess_, ModuleIdent, ModuleIdent_, Value_, Visibility,
    },
    ice,
    naming::ast::DatatypeTypeParameter,
    naming::ast::{self as N, BlockLabel, TParam, TParamID, Type, TypeName_, Type_},
    parser::ast::{
        Ability_, BinOp, BinOp_, ConstantName, DatatypeName, Field, FunctionName, UnaryOp_,
        VariantName,
    },
    shared::{
        known_attributes::TestingAttribute,
        process_binops,
        program_info::{DatatypeKind, TypingProgramInfo},
        unique_map::UniqueMap,
        *,
    },
    sui_mode,
    typing::{
        ast as T,
        core::{make_tvar, public_testing_visibility, PublicForTesting, ResolvedFunctionType},
        dependency_ordering, macro_expand,
    },
    FullyCompiledProgram,
};
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(
    compilation_env: &mut CompilationEnv,
    pre_compiled_lib: Option<&FullyCompiledProgram>,
    prog: N::Program,
) -> T::Program {
    let N::Program {
        info,
        inner: N::Program_ { modules: nmodules },
    } = prog;
    let mut context = Box::new(Context::new(compilation_env, pre_compiled_lib, info));

    extract_macros(&mut context, &nmodules);
    let mut modules = modules(&mut context, nmodules);

    assert!(context.constraints.is_empty());
    dependency_ordering::program(context.env, &mut modules);
    recursive_datatypes::modules(context.env, &modules);
    infinite_instantiations::modules(context.env, &modules);
    let mut prog = T::Program_ { modules };
    // we extract module use funs into the module info context
    let module_use_funs = context
        .modules
        .modules
        .into_iter()
        .map(|(mident, minfo)| (mident, minfo.use_funs))
        .collect();
    let module_info = TypingProgramInfo::new(pre_compiled_lib, &prog, module_use_funs);
    for v in &compilation_env.visitors().typing {
        let mut v = v.borrow_mut();
        v.visit(compilation_env, &module_info, &mut prog);
    }
    T::Program {
        info: module_info,
        inner: prog,
    }
}

fn extract_macros(context: &mut Context, modules: &UniqueMap<ModuleIdent, N::ModuleDefinition>) {
    // Merges the methods of the module into the local methods for each macro.
    fn merge_use_funs(module_use_funs: &N::UseFuns, mut macro_use_funs: N::UseFuns) -> N::UseFuns {
        let N::UseFuns {
            color: _,
            resolved,
            implicit_candidates,
        } = module_use_funs;
        for (tn, module_methods) in resolved {
            let macro_methods = macro_use_funs.resolved.entry(tn.clone()).or_default();
            for (name, method) in module_methods.key_cloned_iter() {
                if !macro_methods.contains_key(&name) {
                    macro_methods.add(name, method.clone()).unwrap();
                }
            }
        }
        for (name, module_candidate) in implicit_candidates.key_cloned_iter() {
            if !macro_use_funs.implicit_candidates.contains_key(&name) {
                macro_use_funs
                    .implicit_candidates
                    .add(name, module_candidate.clone())
                    .unwrap();
            }
        }
        macro_use_funs
    }
    let all_macro_definitions = modules.ref_map(|_mident, mdef| {
        mdef.functions.ref_filter_map(|_name, f| {
            let _macro_loc = f.macro_?;
            if let N::FunctionBody_::Defined((use_funs, body)) = &f.body.value {
                let use_funs = merge_use_funs(&mdef.use_funs, use_funs.clone());
                Some((use_funs, body.clone()))
            } else {
                None
            }
        })
    });

    context.set_macros(all_macro_definitions);
}

fn modules(
    context: &mut Context,
    modules: UniqueMap<ModuleIdent, N::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, T::ModuleDefinition> {
    let mut all_new_friends = BTreeMap::new();
    let mut typed_modules = modules.map(|ident, mdef| {
        let (typed_mdef, new_friends) = module(context, ident, mdef);
        for (pub_package_module, loc) in new_friends {
            let friend = Friend {
                attributes: UniqueMap::new(),
                loc,
            };
            all_new_friends
                .entry(pub_package_module)
                .or_insert_with(BTreeMap::new)
                .insert(ident, friend);
        }
        typed_mdef
    });

    for (mident, friends) in all_new_friends {
        let mdef = typed_modules.get_mut(&mident).unwrap();
        // point of interest: if we have any new friends, we know there can't be any
        // "current" friends becahse all thew new friends are generated off of
        // `public(package)` usage, which disallows other friends.
        mdef.friends = UniqueMap::maybe_from_iter(friends.into_iter())
            .expect("ICE compiler added duplicate friends to public(package) friend list");
    }

    for (_, mident, mdef) in &typed_modules {
        unused_module_members(context, mident, mdef);
    }

    typed_modules
}

fn module(
    context: &mut Context,
    ident: ModuleIdent,
    mdef: N::ModuleDefinition,
) -> (T::ModuleDefinition, BTreeSet<(ModuleIdent, Loc)>) {
    assert!(context.current_package.is_none());
    assert!(context.new_friends.is_empty());

    let N::ModuleDefinition {
        loc,
        warning_filter,
        package_name,
        attributes,
        is_source_module,
        use_funs,
        friends,
        mut structs,
        mut enums,
        functions: nfunctions,
        constants: nconstants,
    } = mdef;
    context.current_module = Some(ident);
    context.current_package = package_name;
    context.env.add_warning_filter_scope(warning_filter.clone());
    context.add_use_funs_scope(use_funs);
    structs
        .iter_mut()
        .for_each(|(_, _, s)| struct_def(context, s));
    enums.iter_mut().for_each(|(_, _, e)| enum_def(context, e));
    process_attributes(context, &attributes);
    let constants = nconstants.map(|name, c| constant(context, name, c));
    let functions = nfunctions.map(|name, f| function(context, name, f));
    assert!(context.constraints.is_empty());
    context.current_package = None;
    let use_funs = context.pop_use_funs_scope();
    context.env.pop_warning_filter_scope();
    let typed_module = T::ModuleDefinition {
        loc,
        warning_filter,
        package_name,
        attributes,
        is_source_module,
        dependency_order: 0,
        immediate_neighbors: UniqueMap::new(),
        used_addresses: BTreeSet::new(),
        use_funs,
        friends,
        structs,
        enums,
        constants,
        functions,
    };
    // get the list of new friends and reset the list.
    let new_friends = std::mem::take(&mut context.new_friends);
    (typed_module, new_friends)
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, name: FunctionName, f: N::Function) -> T::Function {
    let N::Function {
        warning_filter,
        index,
        attributes,
        visibility,
        entry,
        macro_,
        mut signature,
        body: n_body,
    } = f;
    context.env.add_warning_filter_scope(warning_filter.clone());
    assert!(context.constraints.is_empty());
    context.reset_for_module_item();
    context.current_function = Some(name);
    context.in_macro_function = macro_.is_some();
    process_attributes(context, &attributes);
    let visibility =
        match public_testing_visibility(context.env, context.current_package, &name, entry) {
            Some(PublicForTesting::Entry(loc)) => Visibility::Public(loc),
            None => visibility,
        };
    function_signature(context, macro_, &signature);
    expand::function_signature(context, &mut signature);

    let body = if macro_.is_some() {
        sp(n_body.loc, T::FunctionBody_::Macro)
    } else {
        let body = function_body(context, n_body);
        unused_let_muts(context);
        body
    };
    context.current_function = None;
    context.in_macro_function = false;
    context.env.pop_warning_filter_scope();
    T::Function {
        warning_filter,
        index,
        attributes,
        visibility,
        entry,
        macro_,
        signature,
        body,
    }
}

fn function_signature(context: &mut Context, macro_: Option<Loc>, sig: &N::FunctionSignature) {
    assert!(context.constraints.is_empty());

    for (mut_, param, param_ty) in &sig.parameters {
        let mut param_ty = param_ty.clone();
        if macro_.is_some() {
            core::give_tparams_all_abilities(&mut param_ty)
        };
        let param_ty = core::instantiate(context, param_ty);
        // TODO we can relax this for macros once we can bind tuples to variables
        context.add_single_type_constraint(
            param_ty.loc,
            "Invalid parameter type",
            param_ty.clone(),
        );
        context.declare_local(*mut_, *param, param_ty);
    }
    let mut return_type = sig.return_type.clone();
    if macro_.is_some() {
        core::give_tparams_all_abilities(&mut return_type)
    };
    context.return_type = Some(core::instantiate(context, return_type));
    core::solve_constraints(context);
}

fn function_body(context: &mut Context, sp!(loc, nb_): N::FunctionBody) -> T::FunctionBody {
    assert!(context.constraints.is_empty());
    let mut b_ = match nb_ {
        N::FunctionBody_::Native => T::FunctionBody_::Native,
        N::FunctionBody_::Defined(es) => {
            let seq = sequence(context, es);
            let ety = sequence_type(&seq);
            let ret_ty = context.return_type.clone().unwrap();
            let (_, seq_items) = &seq;
            let sloc = seq_items.back().unwrap().loc;
            subtype(
                context,
                sloc,
                || "Invalid return expression",
                ety.clone(),
                ret_ty,
            );
            T::FunctionBody_::Defined(seq)
        }
    };
    core::solve_constraints(context);
    expand::function_body_(context, &mut b_);
    // freeze::function_body_(context, &mut b_);
    sp(loc, b_)
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

fn constant(context: &mut Context, _name: ConstantName, nconstant: N::Constant) -> T::Constant {
    assert!(context.constraints.is_empty());
    context.reset_for_module_item();

    let N::Constant {
        warning_filter,
        index,
        attributes,
        loc,
        signature,
        value: nvalue,
    } = nconstant;
    context.env.add_warning_filter_scope(warning_filter.clone());

    process_attributes(context, &attributes);

    // Don't need to add base type constraint, as it is checked in `check_valid_constant::signature`
    let mut signature = core::instantiate(context, signature);
    check_valid_constant::signature(
        context,
        signature.loc,
        || "Unpermitted constant type",
        TypeSafety::TypeForConstant,
        &signature,
    );
    context.return_type = Some(signature.clone());

    let mut value = exp(context, Box::new(nvalue));
    unused_let_muts(context);
    subtype(
        context,
        signature.loc,
        || "Invalid constant signature",
        value.ty.clone(),
        signature.clone(),
    );
    core::solve_constraints(context);

    expand::type_(context, &mut signature);
    expand::exp(context, &mut value);

    check_valid_constant::exp(context, &value);
    context.env.pop_warning_filter_scope();

    T::Constant {
        warning_filter,
        index,
        attributes,
        loc,
        signature,
        value: *value,
    }
}

mod check_valid_constant {
    use super::subtype_no_report;
    use crate::{
        diag,
        diagnostics::codes::DiagnosticCode,
        naming::ast::{Type, Type_},
        shared::*,
        typing::{
            ast as T,
            core::{self, Context, Subst},
        },
    };
    use move_ir_types::location::*;

    pub(crate) fn signature<T: ToString, F: FnOnce() -> T>(
        context: &mut Context,
        sloc: Loc,
        fmsg: F,
        code: impl DiagnosticCode,
        ty: &Type,
    ) {
        let loc = ty.loc;

        let mut acceptable_types = vec![
            Type_::u8(loc),
            Type_::u16(loc),
            Type_::u32(loc),
            Type_::u64(loc),
            Type_::u128(loc),
            Type_::u256(loc),
            Type_::bool(loc),
            Type_::address(loc),
        ];
        let ty_is_an_acceptable_type = acceptable_types.iter().any(|acceptable_type| {
            let old_subst = context.subst.clone();
            let result = subtype_no_report(context, ty.clone(), acceptable_type.clone());
            context.subst = old_subst;
            result.is_ok()
        });
        if ty_is_an_acceptable_type {
            return;
        }

        let inner_tvar = core::make_tvar(context, sloc);
        let vec_ty = Type_::vector(sloc, inner_tvar.clone());
        let old_subst = context.subst.clone();
        let is_vec = subtype_no_report(context, ty.clone(), vec_ty.clone()).is_ok();
        let inner = core::ready_tvars(&context.subst, inner_tvar);
        context.subst = old_subst;
        if is_vec {
            signature(context, sloc, fmsg, code, &inner);
            return;
        }

        acceptable_types.push(vec_ty);
        let tys = acceptable_types
            .iter()
            .map(|t| core::error_format(t, &Subst::empty()));
        let tmsg = format!(
            "Found: {}. But expected one of: {}",
            core::error_format(ty, &Subst::empty()),
            format_comma(tys),
        );
        context
            .env
            .add_diag(diag!(code, (sloc, fmsg()), (loc, tmsg)))
    }

    pub fn exp(context: &mut Context, e: &T::Exp) {
        exp_(context, &e.exp)
    }

    fn exp_(context: &mut Context, sp!(loc, e_): &T::UnannotatedExp) {
        use T::UnannotatedExp_ as E;
        const REFERENCE_CASE: &str = "References (and reference operations) are";
        let s;
        let error_case = match e_ {
            //*****************************************
            // Error cases handled elsewhere
            //*****************************************
            E::Use(_) | E::Continue(_) | E::Give(_, _) | E::UnresolvedError => return,

            //*****************************************
            // Valid cases
            //*****************************************
            E::Unit { .. } | E::Value(_) | E::Move { .. } | E::Copy { .. } => return,
            E::Block(seq) => {
                sequence(context, seq);
                return;
            }
            E::UnaryExp(_, er) => {
                exp(context, er);
                return;
            }
            E::BinopExp(el, _, _, er) => {
                exp(context, el);
                exp(context, er);
                return;
            }
            E::Cast(el, _) | E::Annotate(el, _) => {
                exp(context, el);
                return;
            }
            E::Vector(_, _, _, eargs) => {
                exp(context, eargs);
                return;
            }
            E::ExpList(el) => {
                exp_list(context, el);
                return;
            }

            // NB: module scoping is checked during constant type creation, so we don't need to
            // relitigate here.
            E::Constant(_, _) => {
                return;
            }

            //*****************************************
            // Invalid cases
            //*****************************************
            E::BorrowLocal(_, _) => REFERENCE_CASE,
            E::ModuleCall(call) => {
                exp(context, &call.arguments);
                "Module calls are"
            }
            E::Builtin(b, args) => {
                exp(context, args);
                s = format!("'{}' is", b);
                &s
            }
            E::IfElse(eb, et, ef) => {
                exp(context, eb);
                exp(context, et);
                exp(context, ef);
                "'if' expressions are"
            }
            E::Match(esubject, sp!(_, arms)) => {
                exp(context, esubject);
                for arm in arms {
                    if let Some(guard) = arm.value.guard.as_ref() {
                        exp(context, guard)
                    }
                    exp(context, &arm.value.rhs);
                }
                "'match' expressions are"
            }
            E::VariantMatch(..) => panic!("ICE shouldn't find variant match before HLIR lowerng"),
            E::While(_, eb, eloop) => {
                exp(context, eb);
                exp(context, eloop);
                "'while' expressions are"
            }
            E::Loop { body: eloop, .. } => {
                exp(context, eloop);
                "'loop' expressions are"
            }
            E::NamedBlock(_, seq) => {
                sequence(context, seq);
                "named 'block' expressions are"
            }
            E::Assign(_assigns, _tys, er) => {
                exp(context, er);
                "Assignments are"
            }
            E::Return(er) => {
                exp(context, er);
                "'return' expressions are"
            }
            E::Abort(er) => {
                exp(context, er);
                "'abort' expressions are"
            }
            E::Dereference(er) | E::Borrow(_, er, _) | E::TempBorrow(_, er) => {
                exp(context, er);
                REFERENCE_CASE
            }
            E::Mutate(el, er) => {
                exp(context, el);
                exp(context, er);
                REFERENCE_CASE
            }
            E::Pack(_, _, _, fields) => {
                for (_, _, (_, (_, fe))) in fields {
                    exp(context, fe)
                }
                "Structs are"
            }
            E::PackVariant(_, _, _, _, fields) => {
                for (_, _, (_, (_, fe))) in fields {
                    exp(context, fe)
                }
                "Enum variants are"
            }
        };
        context.env.add_diag(diag!(
            TypeSafety::UnsupportedConstant,
            (*loc, format!("{} not supported in constants", error_case))
        ));
    }

    fn exp_list(context: &mut Context, items: &[T::ExpListItem]) {
        for item in items {
            exp_list_item(context, item)
        }
    }

    fn exp_list_item(context: &mut Context, item: &T::ExpListItem) {
        use T::ExpListItem as I;
        match item {
            I::Single(e, _st) => {
                exp(context, e);
            }
            I::Splat(_, e, _ss) => {
                exp(context, e);
            }
        }
    }

    fn sequence(context: &mut Context, (_, seq): &T::Sequence) {
        for item in seq {
            sequence_item(context, item)
        }
    }

    fn sequence_item(context: &mut Context, sp!(loc, item_): &T::SequenceItem) {
        use T::SequenceItem_ as S;
        let error_case = match &item_ {
            S::Seq(te) => {
                exp(context, te);
                return;
            }

            S::Declare(_) => "'let' declarations",
            S::Bind(_, _, te) => {
                exp(context, te);
                "'let' declarations"
            }
        };
        let msg = format!("{} are not supported in constants", error_case);
        context
            .env
            .add_diag(diag!(TypeSafety::UnsupportedConstant, (*loc, msg),))
    }
}

//**************************************************************************************************
// Data Types
//**************************************************************************************************

fn struct_def(context: &mut Context, s: &mut N::StructDefinition) {
    assert!(context.constraints.is_empty());
    context.reset_for_module_item();
    context
        .env
        .add_warning_filter_scope(s.warning_filter.clone());

    let field_map = match &mut s.fields {
        N::StructFields::Native(_) => return,
        N::StructFields::Defined(m) => m,
    };

    // instantiate types and check constraints
    for (_field_loc, _field, idx_ty) in field_map.iter() {
        let loc = idx_ty.1.loc;
        let inst_ty = core::instantiate(context, idx_ty.1.clone());
        context.add_base_type_constraint(loc, "Invalid field type", inst_ty.clone());
    }
    core::solve_constraints(context);

    // substitute the declared type parameters with an Any type to check for ability field
    // requirements
    let declared_abilities = &s.abilities;
    let tparam_subst = &core::make_tparam_subst(
        s.type_parameters.iter().map(|tp| &tp.param),
        s.type_parameters
            .iter()
            .map(|tp| sp(tp.param.user_specified_name.loc, Type_::Anything)),
    );
    for (_field_loc, _field, idx_ty) in field_map.iter() {
        let loc = idx_ty.1.loc;
        let subst_ty = core::subst_tparams(tparam_subst, idx_ty.1.clone());
        for declared_ability in declared_abilities {
            let required = declared_ability.value.requires();
            let msg = format!(
                "Invalid field type. The struct was declared with the ability '{}' so all fields \
                 require the ability '{}'",
                declared_ability, required
            );
            context.add_ability_constraint(loc, Some(msg), subst_ty.clone(), required)
        }
    }
    core::solve_constraints(context);

    for (_field_loc, _field_, idx_ty) in field_map.iter_mut() {
        expand::type_(context, &mut idx_ty.1);
    }
    check_type_params_usage(context, &s.type_parameters, field_map);
    context.env.pop_warning_filter_scope();
}

fn enum_def(context: &mut Context, enum_: &mut N::EnumDefinition) {
    assert!(context.constraints.is_empty());

    context
        .env
        .add_warning_filter_scope(enum_.warning_filter.clone());

    let enum_abilities = &enum_.abilities;
    let enum_type_params = &enum_.type_parameters;

    let mut field_types = vec![];
    for (_, _, variant) in enum_.variants.iter_mut() {
        let mut varient_fields = variant_def(context, enum_abilities, enum_type_params, variant);
        field_types.append(&mut varient_fields);
    }

    check_variant_type_params_usage(context, enum_type_params, field_types);
    context.env.pop_warning_filter_scope();
}

fn variant_def(
    context: &mut Context,
    enum_abilities: &AbilitySet,
    enum_tparams: &[DatatypeTypeParameter],
    v: &mut N::VariantDefinition,
) -> Vec<(usize, Type)> {
    context.reset_for_module_item();

    let field_map = match &mut v.fields {
        N::VariantFields::Empty => return vec![],
        N::VariantFields::Defined(m) => m,
    };

    // instantiate types and check constraints
    for (_field_loc, _field, idx_ty) in field_map.iter() {
        let loc = idx_ty.1.loc;
        let inst_ty = core::instantiate(context, idx_ty.1.clone());
        context.add_base_type_constraint(loc, "Invalid field type", inst_ty.clone());
    }
    core::solve_constraints(context);

    // substitute the declared type parameters with an Any type to check for ability field
    // requirements
    let tparam_subst = &core::make_tparam_subst(
        enum_tparams.iter().map(|tp| &tp.param),
        enum_tparams
            .iter()
            .map(|tp| sp(tp.param.user_specified_name.loc, Type_::Anything)),
    );
    for (_field_loc, _field, idx_ty) in field_map.iter() {
        let loc = idx_ty.1.loc;
        let subst_ty = core::subst_tparams(tparam_subst, idx_ty.1.clone());
        for declared_ability in enum_abilities {
            let required = declared_ability.value.requires();
            let msg = format!(
                "Invalid field type. The struct was declared with the ability '{}' so all fields \
                 require the ability '{}'",
                declared_ability, required
            );
            context.add_ability_constraint(loc, Some(msg), subst_ty.clone(), required)
        }
    }
    core::solve_constraints(context);

    for (_field_loc, _field_, idx_ty) in field_map.iter_mut() {
        expand::type_(context, &mut idx_ty.1);
    }
    field_map
        .into_iter()
        .map(|(_, _, idx_ty)| idx_ty.clone())
        .collect::<Vec<_>>()
}

fn check_type_params_usage(
    context: &mut Context,
    type_parameters: &[N::DatatypeTypeParameter],
    field_map: &Fields<Type>,
) {
    let has_unresolved = field_map
        .iter()
        .any(|(_, _, ty)| has_unresolved_error_type(&ty.1));

    if has_unresolved {
        return;
    }

    // true = used at least once in non-phantom pos
    // false = only used in phantom pos
    // not in the map = never used
    let mut non_phantom_use: BTreeMap<TParamID, bool> = BTreeMap::new();
    let phantom_params: BTreeSet<TParamID> = type_parameters
        .iter()
        .filter(|ty_param| ty_param.is_phantom)
        .map(|param| param.param.id)
        .collect();
    for (_, _, idx_ty) in field_map.iter() {
        visit_type_params(
            context,
            &idx_ty.1,
            ParamPos::FIELD,
            &mut |context, loc, param, pos| {
                let param_is_phantom = phantom_params.contains(&param.id);
                match (pos, param_is_phantom) {
                    (ParamPos::NonPhantom(non_phantom_pos), true) => {
                        invalid_phantom_use_error(context, non_phantom_pos, param, loc);
                    }
                    (_, false) => {
                        let used_in_non_phantom_pos =
                            non_phantom_use.entry(param.id).or_insert(false);
                        *used_in_non_phantom_pos |= !pos.is_phantom();
                    }
                    _ => {}
                }
            },
        );
    }
    for ty_param in type_parameters {
        if !ty_param.is_phantom {
            check_non_phantom_param_usage(
                context,
                &ty_param.param,
                non_phantom_use.get(&ty_param.param.id).copied(),
            );
        }
    }
}

fn check_variant_type_params_usage(
    context: &mut Context,
    type_parameters: &[N::DatatypeTypeParameter],
    field_map: Vec<(usize, Type)>,
) {
    let has_unresolved = field_map
        .iter()
        .any(|(_, ty)| has_unresolved_error_type(ty));

    if has_unresolved {
        return;
    }

    // true = used at least once in non-phantom pos
    // false = only used in phantom pos
    // not in the map = never used
    let mut non_phantom_use: BTreeMap<TParamID, bool> = BTreeMap::new();
    let phantom_params: BTreeSet<TParamID> = type_parameters
        .iter()
        .filter(|ty_param| ty_param.is_phantom)
        .map(|param| param.param.id)
        .collect();
    for idx_ty in field_map.iter() {
        visit_type_params(
            context,
            &idx_ty.1,
            ParamPos::FIELD,
            &mut |context, loc, param, pos| {
                let param_is_phantom = phantom_params.contains(&param.id);
                match (pos, param_is_phantom) {
                    (ParamPos::NonPhantom(non_phantom_pos), true) => {
                        invalid_phantom_use_error(context, non_phantom_pos, param, loc);
                    }
                    (_, false) => {
                        let used_in_non_phantom_pos =
                            non_phantom_use.entry(param.id).or_insert(false);
                        *used_in_non_phantom_pos |= !pos.is_phantom();
                    }
                    _ => {}
                }
            },
        );
    }
    for ty_param in type_parameters {
        if !ty_param.is_phantom {
            check_non_phantom_param_usage(
                context,
                &ty_param.param,
                non_phantom_use.get(&ty_param.param.id).copied(),
            );
        }
    }
}

#[derive(Clone, Copy)]
enum ParamPos {
    Phantom,
    NonPhantom(NonPhantomPos),
}

impl ParamPos {
    const FIELD: ParamPos = ParamPos::NonPhantom(NonPhantomPos::FieldType);

    /// Returns `true` if the param_pos is [`Phantom`].
    fn is_phantom(&self) -> bool {
        matches!(self, Self::Phantom)
    }
}

#[derive(Clone, Copy)]
enum NonPhantomPos {
    FieldType,
    TypeArg,
}

fn visit_type_params(
    context: &mut Context,
    ty: &Type,
    param_pos: ParamPos,
    f: &mut impl FnMut(&mut Context, Loc, &TParam, ParamPos),
) {
    match &ty.value {
        Type_::Param(param) => {
            f(context, ty.loc, param, param_pos);
        }
        // References cannot appear in structs, but we still report them as a non-phantom position
        // for full information.
        Type_::Ref(_, ty) => {
            visit_type_params(context, ty, ParamPos::NonPhantom(NonPhantomPos::TypeArg), f)
        }
        Type_::Apply(_, n, ty_args) => match &n.value {
            // Tuples cannot appear in structs, but we still report them as a non-phantom position
            // for full information.
            TypeName_::Builtin(_) | TypeName_::Multiple(_) => {
                for ty_arg in ty_args {
                    visit_type_params(
                        context,
                        ty_arg,
                        ParamPos::NonPhantom(NonPhantomPos::TypeArg),
                        f,
                    );
                }
            }
            TypeName_::ModuleType(m, n) => {
                let tparams = match context.datatype_kind(m, n) {
                    DatatypeKind::Enum => context.enum_tparams(m, n),
                    DatatypeKind::Struct => context.struct_tparams(m, n),
                };
                let param_is_phantom: Vec<_> = tparams.iter().map(|p| p.is_phantom).collect();
                // Length of params and args may be different but we can still report errors
                // for parameters with information
                for (is_phantom, ty_arg) in param_is_phantom.into_iter().zip(ty_args) {
                    let pos = if is_phantom {
                        ParamPos::Phantom
                    } else {
                        ParamPos::NonPhantom(NonPhantomPos::TypeArg)
                    };
                    visit_type_params(context, ty_arg, pos, f);
                }
            }
        },
        Type_::Fun(args, result) => {
            for ty in args {
                visit_type_params(context, ty, ParamPos::NonPhantom(NonPhantomPos::TypeArg), f)
            }
            visit_type_params(
                context,
                result,
                ParamPos::NonPhantom(NonPhantomPos::TypeArg),
                f,
            )
        }
        Type_::Var(_) | Type_::Anything | Type_::UnresolvedError => {}
        Type_::Unit => {}
    }
}

fn invalid_phantom_use_error(
    context: &mut Context,
    non_phantom_pos: NonPhantomPos,
    param: &TParam,
    ty_loc: Loc,
) {
    let msg = match non_phantom_pos {
        NonPhantomPos::FieldType => "Phantom type parameter cannot be used as a field type",
        NonPhantomPos::TypeArg => {
            "Phantom type parameter cannot be used as an argument to a non-phantom parameter"
        }
    };
    let decl_msg = format!("'{}' declared here as phantom", &param.user_specified_name);
    context.env.add_diag(diag!(
        Declarations::InvalidPhantomUse,
        (ty_loc, msg),
        (param.user_specified_name.loc, decl_msg),
    ));
}

fn check_non_phantom_param_usage(
    context: &mut Context,
    param: &N::TParam,
    param_usage: Option<bool>,
) {
    let name = &param.user_specified_name;
    match param_usage {
        None => {
            let msg = format!(
                "Unused type parameter '{}'. Consider declaring it as phantom",
                name
            );
            context
                .env
                .add_diag(diag!(UnusedItem::StructTypeParam, (name.loc, msg)))
        }
        Some(false) => {
            let msg = format!(
                "The parameter '{}' is only used as an argument to phantom parameters. Consider \
                 adding a phantom declaration here",
                name
            );
            context
                .env
                .add_diag(diag!(Declarations::InvalidNonPhantomUse, (name.loc, msg)))
        }
        Some(true) => {}
    }
}

fn has_unresolved_error_type(ty: &Type) -> bool {
    match &ty.value {
        Type_::UnresolvedError => true,
        Type_::Ref(_, ty) => has_unresolved_error_type(ty),
        Type_::Apply(_, _, ty_args) => ty_args.iter().any(has_unresolved_error_type),
        Type_::Fun(args, result) => {
            args.iter().any(has_unresolved_error_type) || has_unresolved_error_type(result)
        }
        Type_::Param(_) | Type_::Var(_) | Type_::Anything | Type_::Unit => false,
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn typing_error<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    from_subtype: bool,
    loc: Loc,
    mk_msg: F,
    e: core::TypingError,
) -> Diagnostic {
    use super::core::TypingError::*;
    let msg = mk_msg().to_string();
    let subst = &context.subst;

    match e {
        SubtypeError(t1, t2) => {
            let loc1 = core::best_loc(subst, &t1);
            let loc2 = core::best_loc(subst, &t2);
            let t1_str = core::error_format(&t1, subst);
            let t2_str = core::error_format(&t2, subst);
            let m1 = format!("Given: {}", t1_str);
            let m2 = format!("Expected: {}", t2_str);
            diag!(TypeSafety::SubtypeError, (loc, msg), (loc1, m1), (loc2, m2))
        }
        ArityMismatch(n1, t1, n2, t2) => {
            let loc1 = core::best_loc(subst, &t1);
            let loc2 = core::best_loc(subst, &t2);
            let t1_str = core::error_format(&t1, subst);
            let t2_str = core::error_format(&t2, subst);
            let msg1 = if from_subtype {
                format!("Given expression list of length {}: {}", n1, t1_str)
            } else {
                format!(
                    "Found expression list of length {}: {}. It is not compatible with the other \
                     type of length {}.",
                    n1, t1_str, n2
                )
            };
            let msg2 = if from_subtype {
                format!("Expected expression list of length {}: {}", n2, t2_str)
            } else {
                format!(
                    "Found expression list of length {}: {}. It is not compatible with the other \
                     type of length {}.",
                    n2, t2_str, n1
                )
            };

            diag!(
                TypeSafety::JoinError,
                (loc, msg),
                (loc1, msg1),
                (loc2, msg2)
            )
        }
        FunArityMismatch(a1, t1, a2, t2) => {
            let loc1 = core::best_loc(subst, &t1);
            let loc2 = core::best_loc(subst, &t2);
            let t1_str = core::error_format(&t1, subst);
            let t2_str = core::error_format(&t2, subst);
            let msg1 = if from_subtype {
                format!("Given lambda with {} arguments: {}", a1, t1_str)
            } else {
                format!(
                    "Found a lambda type with {} arguments: {}. It is not compatible with the \
                     other type with {} arguments.",
                    a1, t1_str, a2
                )
            };
            let msg2 = if from_subtype {
                format!("Expected a lambda with {} arguments: {}", a2, t2_str)
            } else {
                format!(
                    "Found a lambda type with {} arguments: {}. It is not compatible with the \
                     other type with {} arguments.",
                    a2, t2_str, a1
                )
            };

            diag!(
                TypeSafety::JoinError,
                (loc, msg),
                (loc1, msg1),
                (loc2, msg2)
            )
        }
        Incompatible(t1, t2) => {
            let loc1 = core::best_loc(subst, &t1);
            let loc2 = core::best_loc(subst, &t2);
            let t1_str = core::error_format(&t1, subst);
            let t2_str = core::error_format(&t2, subst);
            let m1 = if from_subtype {
                format!("Given: {}", t1_str)
            } else {
                format!(
                    "Found: {}. It is not compatible with the other type.",
                    t1_str
                )
            };
            let m2 = if from_subtype {
                format!("Expected: {}", t2_str)
            } else {
                format!(
                    "Found: {}. It is not compatible with the other type.",
                    t2_str
                )
            };
            diag!(TypeSafety::JoinError, (loc, msg), (loc1, m1), (loc2, m2))
        }
        RecursiveType(rloc) => diag!(
            TypeSafety::RecursiveType,
            (loc, msg),
            (rloc, "Unable to infer the type. Recursive type found."),
        ),
    }
}

fn subtype_no_report(
    context: &mut Context,
    pre_lhs: Type,
    pre_rhs: Type,
) -> Result<Type, core::TypingError> {
    let subst = std::mem::replace(&mut context.subst, Subst::empty());
    let lhs = core::ready_tvars(&subst, pre_lhs);
    let rhs = core::ready_tvars(&subst, pre_rhs);
    core::subtype(subst, &lhs, &rhs).map(|(next_subst, ty)| {
        context.subst = next_subst;
        ty
    })
}

fn subtype_impl<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    pre_lhs: Type,
    pre_rhs: Type,
) -> Result<Type, Type> {
    let subst = std::mem::replace(&mut context.subst, Subst::empty());
    let lhs = core::ready_tvars(&subst, pre_lhs);
    let rhs = core::ready_tvars(&subst, pre_rhs);
    match core::subtype(subst.clone(), &lhs, &rhs) {
        Err(e) => {
            context.subst = subst;
            let diag = typing_error(context, /* from_subtype */ true, loc, msg, e);
            context.env.add_diag(diag);
            Err(rhs)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            Ok(ty)
        }
    }
}

fn subtype_opt<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    pre_lhs: Type,
    pre_rhs: Type,
) -> Option<Type> {
    match subtype_impl(context, loc, msg, pre_lhs, pre_rhs) {
        Err(_rhs) => None,
        Ok(t) => Some(t),
    }
}

fn subtype<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    pre_lhs: Type,
    pre_rhs: Type,
) -> Type {
    match subtype_impl(context, loc, msg, pre_lhs, pre_rhs) {
        Err(rhs) => rhs,
        Ok(t) => t,
    }
}

fn join_opt<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    pre_t1: Type,
    pre_t2: Type,
) -> Option<Type> {
    let subst = std::mem::replace(&mut context.subst, Subst::empty());
    let t1 = core::ready_tvars(&subst, pre_t1);
    let t2 = core::ready_tvars(&subst, pre_t2);
    match core::join(subst.clone(), &t1, &t2) {
        Err(e) => {
            context.subst = subst;
            let diag = typing_error(context, /* from_subtype */ false, loc, msg, e);
            context.env.add_diag(diag);
            None
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            Some(ty)
        }
    }
}

fn join<T: ToString, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    pre_t1: Type,
    pre_t2: Type,
) -> Type {
    match join_opt(context, loc, msg, pre_t1, pre_t2) {
        None => context.error_type(loc),
        Some(ty) => ty,
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

enum SeqCase {
    Seq(Loc, Box<T::Exp>),
    Declare {
        loc: Loc,
        b: T::LValueList,
    },
    Bind {
        loc: Loc,
        b: T::LValueList,
        e: Box<T::Exp>,
    },
}

fn sequence(context: &mut Context, (use_funs, seq): N::Sequence) -> T::Sequence {
    use N::SequenceItem_ as NS;
    use T::SequenceItem_ as TS;

    context.add_use_funs_scope(use_funs);
    let mut work_queue = VecDeque::new();

    let len = seq.len();
    for (idx, sp!(loc, ns_)) in seq.into_iter().enumerate() {
        match ns_ {
            NS::Seq(ne) => {
                let e = exp(context, ne);
                // If it is not the last element
                if idx < len - 1 {
                    context.add_ability_constraint(
                        loc,
                        Some(format!(
                            "Cannot ignore values without the '{}' ability. The value must be used",
                            Ability_::Drop
                        )),
                        e.ty.clone(),
                        Ability_::Drop,
                    )
                }
                work_queue.push_front(SeqCase::Seq(loc, e));
            }
            NS::Declare(nbind, ty_opt) => {
                let instantiated_ty_op = ty_opt.map(|t| core::instantiate(context, t));
                let b = bind_list(context, nbind, instantiated_ty_op);
                work_queue.push_front(SeqCase::Declare { loc, b });
            }
            NS::Bind(nbind, nr) => {
                let e = exp(context, nr);
                let b = bind_list(context, nbind, Some(e.ty.clone()));
                work_queue.push_front(SeqCase::Bind { loc, b, e });
            }
        }
    }

    let mut seq_items = VecDeque::new();
    for case in work_queue {
        match case {
            SeqCase::Seq(loc, e) => seq_items.push_front(sp(loc, TS::Seq(e))),
            SeqCase::Declare { loc, b } => seq_items.push_front(sp(loc, TS::Declare(b))),
            SeqCase::Bind { loc, b, e } => {
                let lvalue_ty = lvalues_expected_types(context, &b);
                seq_items.push_front(sp(loc, TS::Bind(b, lvalue_ty, e)))
            }
        }
    }
    let use_funs = context.pop_use_funs_scope();
    (use_funs, seq_items)
}

fn sequence_type((_, seq): &T::Sequence) -> &Type {
    use T::SequenceItem_ as TS;
    match seq.back().unwrap() {
        sp!(_, TS::Bind(_, _, _)) | sp!(_, TS::Declare(_)) => {
            panic!("ICE unit should have been inserted past bind/decl")
        }
        sp!(_, TS::Seq(last_e)) => &last_e.ty,
    }
}

fn exp_vec(context: &mut Context, es: Vec<N::Exp>) -> Vec<T::Exp> {
    es.into_iter().map(|e| *exp(context, Box::new(e))).collect()
}

fn exp(context: &mut Context, ne: Box<N::Exp>) -> Box<T::Exp> {
    use N::Exp_ as NE;
    use T::UnannotatedExp_ as TE;
    if matches!(ne.value, NE::BinopExp(..)) {
        return process_binops!(
            (BinOp, Loc),
            Box<T::Exp>,
            *ne,
            sp!(loc, cur_),
            cur_,
            NE::BinopExp(lhs, op, rhs) => { (*lhs, (op, loc), *rhs) },
            { exp(context, Box::new(sp(loc, cur_))) },
            value_stack,
            (bop, loc) => {
                let el = value_stack.pop().expect("ICE binop typing issue");
                let er = value_stack.pop().expect("ICE binop typing issue");
                binop(context, el, bop, loc, er)
            }
        );
    }

    let sp!(eloc, ne_) = *ne;
    let (ty, e_) = match ne_ {
        NE::Unit { trailing } => (sp(eloc, Type_::Unit), TE::Unit { trailing }),
        NE::Value(sp!(vloc, Value_::InferredNum(v))) => (
            core::make_num_tvar(context, eloc),
            TE::Value(sp(vloc, Value_::InferredNum(v))),
        ),
        NE::Value(sp!(vloc, v)) => (v.type_(vloc).unwrap(), TE::Value(sp(vloc, v))),

        NE::Constant(m, c) => {
            let ty = core::make_constant_type(context, eloc, &m, &c);
            context
                .used_module_members
                .entry(m.value)
                .or_default()
                .insert(c.value());
            (ty, TE::Constant(m, c))
        }

        NE::Var(var) => {
            let ty = context.get_local_type(&var);
            (ty, TE::Use(var))
        }
        NE::MethodCall(ndotted, f, /* is_macro */ None, ty_args_opt, sp!(argloc, nargs_)) => {
            let (edotted, last_ty) = exp_dotted(context, None, ndotted);
            let args = exp_vec(context, nargs_);
            let ty_call_opt = method_call(
                context,
                eloc,
                edotted,
                last_ty,
                f,
                ty_args_opt,
                argloc,
                args,
            );
            match ty_call_opt {
                None => {
                    assert!(context.env.has_errors());
                    (context.error_type(eloc), TE::UnresolvedError)
                }
                Some(ty_call) => ty_call,
            }
        }
        NE::ModuleCall(m, f, /* is_macro */ None, ty_args_opt, sp!(argloc, nargs_)) => {
            let args = exp_vec(context, nargs_);
            module_call(context, eloc, m, f, ty_args_opt, argloc, args)
        }
        NE::MethodCall(ndotted, f, Some(macro_call_loc), ty_args_opt, sp!(argloc, nargs_)) => {
            let (edotted, last_ty) = exp_dotted(context, None, ndotted);
            let ty_call_opt = macro_method_call(
                context,
                eloc,
                edotted,
                last_ty,
                f,
                macro_call_loc,
                ty_args_opt,
                argloc,
                nargs_,
            );
            match ty_call_opt {
                None => {
                    assert!(context.env.has_errors());
                    (context.error_type(eloc), TE::UnresolvedError)
                }
                Some(ty_call) => ty_call,
            }
        }
        NE::ModuleCall(m, f, Some(macro_call_loc), ty_args_opt, sp!(argloc, nargs_)) => {
            macro_module_call(
                context,
                eloc,
                m,
                f,
                macro_call_loc,
                ty_args_opt,
                argloc,
                nargs_,
            )
        }
        NE::VarCall(_, sp!(_, nargs_)) => {
            exp_vec(context, nargs_);
            assert!(
                context.env.has_errors(),
                "ICE unbound var call. Should be expanded"
            );
            (context.error_type(eloc), TE::UnresolvedError)
        }
        NE::Builtin(b, sp!(argloc, nargs_)) => {
            let args = exp_vec(context, nargs_);
            builtin_call(context, eloc, b, argloc, args)
        }
        NE::Vector(vec_loc, ty_opt, sp!(argloc, nargs_)) => {
            let args_ = exp_vec(context, nargs_);
            vector_pack(context, eloc, vec_loc, ty_opt, argloc, args_)
        }

        NE::IfElse(nb, nt, nf) => {
            let eb = exp(context, nb);
            let bloc = eb.exp.loc;
            subtype(
                context,
                bloc,
                || "Invalid if condition",
                eb.ty.clone(),
                Type_::bool(bloc),
            );
            let et = exp(context, nt);
            let ef = exp(context, nf);
            let ty = join(
                context,
                eloc,
                || "Incompatible branches",
                et.ty.clone(),
                ef.ty.clone(),
            );
            (ty, TE::IfElse(eb, et, ef))
        }
        NE::Match(nsubject, sp!(aloc, narms_)) => {
            let esubject = exp(context, nsubject);
            let subject_type = core::unfold_type(&context.subst, esubject.ty.clone());
            let ref_mut = match subject_type.value {
                Type_::Ref(mut_, _) => Some(mut_),
                _ => {
                    // Do not need base constraint because of the joins in `match_arms`.
                    None
                }
            };
            let result_type = core::make_tvar(context, aloc);
            let earms = match_arms(context, &subject_type, &result_type, narms_, &ref_mut);
            (result_type, TE::Match(esubject, sp(aloc, earms)))
        }
        NE::While(name, nb, nloop) => {
            let eb = exp(context, nb);
            let bloc = eb.exp.loc;
            subtype(
                context,
                bloc,
                || "Invalid while condition",
                eb.ty.clone(),
                Type_::bool(bloc),
            );
            let (_has_break, ty, body) = loop_body(context, eloc, name, false, nloop);
            (sp(eloc, ty.value), TE::While(name, eb, body))
        }
        NE::Loop(name, nloop) => {
            let (has_break, ty, body) = loop_body(context, eloc, name, true, nloop);
            let eloop = TE::Loop {
                name,
                has_break,
                body,
            };
            (sp(eloc, ty.value), eloop)
        }
        NE::Block(N::Block {
            name,
            from_macro_argument,
            seq: nseq,
        }) => {
            context.maybe_enter_macro_argument(from_macro_argument, nseq.0.color);
            let seq = sequence(context, nseq);
            let seq_ty = sequence_type(&seq).clone();
            let res = if let Some(name) = name {
                let final_type = if let Some(local_return_type) = context.named_block_type_opt(name)
                {
                    let msg = if let Some(N::MacroArgument::Lambda(_)) = from_macro_argument {
                        || "Invalid lambda return"
                    } else {
                        || "Invalid named block"
                    };
                    join(context, eloc, msg, seq_ty, local_return_type)
                } else {
                    seq_ty
                };
                (sp(eloc, final_type.value), TE::NamedBlock(name, seq))
            } else {
                (seq_ty, TE::Block(seq))
            };
            context.maybe_exit_macro_argument(eloc, from_macro_argument);
            res
        }

        NE::Lambda(_) => {
            if context
                .env
                .check_feature(FeatureGate::MacroFuns, context.current_package, eloc)
            {
                let msg = "Lambdas can only be used directly as arguments to 'macro' functions";
                context
                    .env
                    .add_diag(diag!(TypeSafety::UnexpectedLambda, (eloc, msg)))
            }
            (context.error_type(eloc), TE::UnresolvedError)
        }

        NE::Assign(na, nr) => {
            let er = exp(context, nr);
            let a = assign_list(context, na, er.ty.clone());
            let lvalue_ty = lvalues_expected_types(context, &a);
            (sp(eloc, Type_::Unit), TE::Assign(a, lvalue_ty, er))
        }

        NE::Mutate(nl, nr) => {
            let el = exp(context, nl);
            let er = exp(context, nr);
            check_mutation(context, el.exp.loc, el.ty.clone(), &er.ty);
            (sp(eloc, Type_::Unit), TE::Mutate(el, er))
        }

        NE::FieldMutate(ndotted, nr) => {
            let lhsloc = ndotted.loc;
            let er = exp(context, nr);
            let (edotted, _) = exp_dotted(context, Some("mutation"), ndotted);
            let eborrow = exp_dotted_to_borrow(context, lhsloc, true, edotted);
            check_mutation(context, eborrow.exp.loc, eborrow.ty.clone(), &er.ty);
            (sp(eloc, Type_::Unit), TE::Mutate(Box::new(eborrow), er))
        }

        NE::Return(nret) => {
            let eret = exp(context, nret);
            let ret_ty = context.return_type.clone().unwrap();
            subtype(context, eloc, || "Invalid return", eret.ty.clone(), ret_ty);
            (sp(eloc, Type_::Anything), TE::Return(eret))
        }
        NE::Abort(ncode) => {
            let ecode = exp(context, ncode);
            let code_ty = Type_::u64(eloc);
            subtype(context, eloc, || "Invalid abort", ecode.ty.clone(), code_ty);
            (sp(eloc, Type_::Anything), TE::Abort(ecode))
        }
        NE::Give(usage, name, rhs) => {
            let break_rhs = exp(context, rhs);
            let loop_ty = context.named_block_type(name, eloc);
            subtype(
                context,
                eloc,
                || format!("Invalid {usage}"),
                break_rhs.ty.clone(),
                loop_ty,
            );
            (sp(eloc, Type_::Anything), TE::Give(name, break_rhs))
        }
        NE::Continue(name) => (sp(eloc, Type_::Anything), TE::Continue(name)),

        NE::Dereference(nref) => {
            let eref = exp(context, nref);
            let inner = core::make_tvar(context, eloc);
            let ref_ty = sp(eloc, Type_::Ref(false, Box::new(inner.clone())));
            subtype(
                context,
                eloc,
                || "Invalid dereference.",
                eref.ty.clone(),
                ref_ty,
            );
            context.add_ability_constraint(
                eloc,
                Some(format!(
                    "Invalid dereference. Dereference requires the '{}' ability",
                    Ability_::Copy
                )),
                inner.clone(),
                Ability_::Copy,
            );
            (inner, TE::Dereference(eref))
        }
        NE::UnaryExp(uop, nr) => {
            use UnaryOp_::*;
            let msg = || format!("Invalid argument to '{}'", &uop);
            let er = exp(context, nr);
            let ty = match &uop.value {
                Not => {
                    let rloc = er.exp.loc;
                    subtype(context, rloc, msg, er.ty.clone(), Type_::bool(rloc));
                    Type_::bool(eloc)
                }
            };
            (ty, TE::UnaryExp(uop, er))
        }

        NE::ExpList(nes) => {
            assert!(!nes.is_empty());
            let es = exp_vec(context, nes);
            let locs = es.iter().map(|e| e.exp.loc).collect();
            let tvars = core::make_expr_list_tvars(
                context,
                eloc,
                "Invalid expression list type argument",
                locs,
            );
            for (e, tvar) in es.iter().zip(&tvars) {
                join(
                    context,
                    e.exp.loc,
                    || -> String { panic!("ICE failed tvar join") },
                    e.ty.clone(),
                    tvar.clone(),
                );
            }
            let ty = Type_::multiple(eloc, tvars);
            let items = es.into_iter().map(T::single_item).collect();
            (ty, TE::ExpList(items))
        }

        NE::Pack(m, n, ty_args_opt, nfields) => {
            let (bt, targs) = core::make_struct_type(context, eloc, &m, &n, ty_args_opt);
            let typed_nfields =
                add_struct_field_types(context, eloc, "argument", &m, &n, targs.clone(), nfields);

            let tfields = typed_nfields.map(|f, (idx, (fty, narg))| {
                let arg = exp(context, Box::new(narg));
                subtype(
                    context,
                    arg.exp.loc,
                    || format!("Invalid argument for field '{}' for '{}::{}'", f, &m, &n),
                    arg.ty.clone(),
                    fty.clone(),
                );
                (idx, (fty, *arg))
            });
            if !context.is_current_module(&m) {
                let msg = format!(
                    "Invalid instantiation of '{}::{}'.\nAll structs can only be constructed in \
                     the module in which they are declared",
                    &m, &n,
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::Visibility, (eloc, msg)));
            }
            (bt, TE::Pack(m, n, targs, tfields))
        }

        NE::PackVariant(m, e, v, ty_args_opt, nfields) => {
            let (bt, targs) = core::make_enum_type(context, eloc, &m, &e, ty_args_opt);
            let typed_nfields = add_variant_field_types(
                context,
                eloc,
                "argument",
                &m,
                &e,
                &v,
                targs.clone(),
                nfields,
            );

            let tfields = typed_nfields.map(|f, (idx, (fty, narg))| {
                let arg = exp(context, Box::new(narg));
                subtype(
                    context,
                    arg.exp.loc,
                    || {
                        format!(
                            "Invalid argument for field '{}' for '{}::{}::{}'",
                            f, &m, &e, &v
                        )
                    },
                    arg.ty.clone(),
                    fty.clone(),
                );
                (idx, (fty, *arg))
            });
            if !context.is_current_module(&m) {
                let msg = format!(
                    "Invalid instantiation of '{}::{}::{}'.\nAll enum variants can only be \
                    constructed in the module in which they are declared",
                    &m, &e, &v
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::Visibility, (eloc, msg)));
            }
            (bt, TE::PackVariant(m, e, v, targs, tfields))
        }

        NE::ExpDotted(DottedUsage::Use, sp!(_, N::ExpDotted_::Exp(ner))) => {
            let er = exp(context, ner);
            (er.ty, er.exp.value)
        }
        NE::ExpDotted(DottedUsage::Borrow(mut_), sp!(_, N::ExpDotted_::Exp(ner))) => {
            let er = exp(context, ner);
            warn_on_constant_borrow(context, eloc, &er);
            context.add_base_type_constraint(eloc, "Invalid borrow", er.ty.clone());
            let ty = sp(eloc, Type_::Ref(mut_, Box::new(er.ty.clone())));
            let eborrow = match er.exp {
                sp!(_, TE::Use(v)) => {
                    if mut_ {
                        check_mutability(context, eloc, "mutable borrow", &v);
                    }
                    TE::BorrowLocal(mut_, v)
                }
                erexp => TE::TempBorrow(mut_, Box::new(T::exp(er.ty, erexp))),
            };
            (ty, eborrow)
        }
        NE::ExpDotted(DottedUsage::Move(loc), sp!(_, N::ExpDotted_::Exp(ner))) => {
            let er = exp(context, ner);

            match er.exp.value {
                TE::Use(var) => (
                    er.ty,
                    TE::Move {
                        var,
                        from_user: true,
                    },
                ),
                TE::UnresolvedError => (er.ty, TE::UnresolvedError),
                er_ => {
                    let msg = if matches!(er_, TE::Constant(_, _)) {
                        "Invalid 'move'. Cannot 'move' constants"
                    } else {
                        "Invalid 'move'. Expected a variable or path."
                    };
                    context
                        .env
                        .add_diag(diag!(TypeSafety::InvalidMoveOp, (loc, msg)));
                    (context.error_type(eloc), TE::UnresolvedError)
                }
            }
        }
        NE::ExpDotted(DottedUsage::Copy(loc), sp!(_, N::ExpDotted_::Exp(ner))) => {
            let er = exp(context, ner);
            let (ty, ecopy) = match er.exp.value {
                TE::Use(var) => (
                    er.ty,
                    TE::Copy {
                        var,
                        from_user: true,
                    },
                ),
                er_ @ TE::Constant(_, _) => {
                    context.env.check_feature(
                        FeatureGate::Move2024Paths,
                        context.current_package(),
                        loc,
                    );
                    (er.ty, er_)
                }
                TE::UnresolvedError => (er.ty, TE::UnresolvedError),
                _ => {
                    let msg = "Invalid 'copy'. Expected a variable or path.".to_owned();
                    context
                        .env
                        .add_diag(diag!(TypeSafety::InvalidCopyOp, (loc, msg)));
                    (context.error_type(eloc), TE::UnresolvedError)
                }
            };
            if !matches!(ecopy, TE::UnresolvedError) {
                context.add_ability_constraint(
                    eloc,
                    Some(format!(
                        "Invalid 'copy' of owned value without the '{}' ability",
                        Ability_::Copy
                    )),
                    ty.clone(),
                    Ability_::Copy,
                );
            }
            (ty, ecopy)
        }

        NE::ExpDotted(DottedUsage::Borrow(mut_), ndotted) => {
            let (edotted, _) = exp_dotted(context, Some("borrow"), ndotted);
            let eborrow = exp_dotted_to_borrow(context, eloc, mut_, edotted);
            (eborrow.ty, eborrow.exp.value)
        }

        NE::ExpDotted(usage, ndotted) => {
            let (edotted, inner_ty) = exp_dotted(context, Some("dot access"), ndotted);
            let ederefborrow = exp_dotted_to_owned_value(context, usage, eloc, edotted, inner_ty);
            (ederefborrow.ty, ederefborrow.exp.value)
        }

        NE::Cast(nl, ty) => {
            let el = exp(context, nl);
            let rhs = core::instantiate(context, ty);
            context.add_numeric_constraint(el.exp.loc, "as", el.ty.clone());
            context.add_numeric_constraint(el.exp.loc, "as", rhs.clone());
            (rhs.clone(), TE::Cast(el, Box::new(rhs)))
        }

        NE::Annotate(nl, ty_annot) => {
            let el = exp(context, nl);
            let annot_loc = ty_annot.loc;
            let msg = || "Invalid type annotation";
            let rhs = core::instantiate(context, ty_annot);
            subtype(context, annot_loc, msg, el.ty.clone(), rhs.clone());
            let e_ = TE::Annotate(el, Box::new(rhs.clone()));
            (rhs, e_)
        }
        NE::UnresolvedError => {
            assert!(context.env.has_errors());
            (context.error_type(eloc), TE::UnresolvedError)
        }

        NE::BinopExp(..) => unreachable!(),
    };
    Box::new(T::exp(ty, sp(eloc, e_)))
}

fn binop(
    context: &mut Context,
    el: Box<T::Exp>,
    bop: BinOp,
    loc: Loc,
    er: Box<T::Exp>,
) -> Box<T::Exp> {
    use BinOp_::*;
    use T::UnannotatedExp_ as TE;
    let msg = || format!("Incompatible arguments to '{}'", &bop);
    let (ty, operand_ty) = match &bop.value {
        Sub | Add | Mul | Mod | Div => {
            context.add_numeric_constraint(el.exp.loc, bop.value.symbol(), el.ty.clone());
            context.add_numeric_constraint(er.exp.loc, bop.value.symbol(), el.ty.clone());
            let operand_ty = join(context, bop.loc, msg, el.ty.clone(), er.ty.clone());
            (operand_ty.clone(), operand_ty)
        }

        BitOr | BitAnd | Xor => {
            context.add_bits_constraint(el.exp.loc, bop.value.symbol(), el.ty.clone());
            context.add_bits_constraint(er.exp.loc, bop.value.symbol(), el.ty.clone());
            let operand_ty = join(context, bop.loc, msg, el.ty.clone(), er.ty.clone());
            (operand_ty.clone(), operand_ty)
        }

        Shl | Shr => {
            let msg = || format!("Invalid argument to '{}'", &bop);
            let u8ty = Type_::u8(er.exp.loc);
            context.add_bits_constraint(el.exp.loc, bop.value.symbol(), el.ty.clone());
            subtype(context, er.exp.loc, msg, er.ty.clone(), u8ty);
            (el.ty.clone(), el.ty.clone())
        }

        Lt | Gt | Le | Ge => {
            context.add_ordered_constraint(el.exp.loc, bop.value.symbol(), el.ty.clone());
            context.add_ordered_constraint(er.exp.loc, bop.value.symbol(), el.ty.clone());
            let operand_ty = join(context, bop.loc, msg, el.ty.clone(), er.ty.clone());
            (Type_::bool(loc), operand_ty)
        }

        Eq | Neq => {
            let ability_msg = Some(format!(
                "'{}' requires the '{}' ability as the value is consumed. Try \
                         borrowing the values with '&' first.'",
                &bop,
                Ability_::Drop,
            ));
            context.add_ability_constraint(
                el.exp.loc,
                ability_msg.clone(),
                el.ty.clone(),
                Ability_::Drop,
            );
            context.add_ability_constraint(er.exp.loc, ability_msg, er.ty.clone(), Ability_::Drop);
            let ty = join(context, bop.loc, msg, el.ty.clone(), er.ty.clone());
            context.add_single_type_constraint(loc, msg(), ty.clone());
            (Type_::bool(loc), ty)
        }

        And | Or => {
            let msg = || format!("Invalid argument to '{}'", &bop);
            let lloc = el.exp.loc;
            subtype(context, lloc, msg, el.ty.clone(), Type_::bool(bop.loc));
            let rloc = er.exp.loc;
            subtype(context, rloc, msg, er.ty.clone(), Type_::bool(bop.loc));
            (Type_::bool(loc), Type_::bool(loc))
        }

        Range | Implies | Iff => {
            context
                .env
                .add_diag(ice!((loc, "ICE unexpect specification operator")));
            (context.error_type(loc), context.error_type(loc))
        }
    };
    Box::new(T::exp(
        ty,
        sp(loc, TE::BinopExp(el, bop, Box::new(operand_ty), er)),
    ))
}

fn loop_body(
    context: &mut Context,
    eloc: Loc,
    name: BlockLabel,
    is_loop: bool,
    nloop: Box<N::Exp>,
) -> (bool, Type, Box<T::Exp>) {
    // set while break to ()
    if !is_loop {
        let while_loop_type = context.named_block_type(name, eloc);
        // while loop breaks must break with unit
        subtype(
            context,
            eloc,
            || "Cannot use 'break' with a non-'()' value in 'while'",
            while_loop_type,
            sp(eloc, Type_::Unit),
        );
    }

    let eloop = exp(context, nloop);
    let lloc = eloop.exp.loc;
    subtype(
        context,
        lloc,
        || "Invalid loop body",
        eloop.ty.clone(),
        sp(lloc, Type_::Unit),
    );

    let break_ty_opt = context.named_block_type_opt(name);

    if let Some(break_ty) = break_ty_opt {
        (true, break_ty, eloop)
    } else {
        // if it was a while loop, the `if` case ran, so we can simply make a type var for the loop
        (false, context.named_block_type(name, eloc), eloop)
    }
}

fn match_arms(
    context: &mut Context,
    subject_type: &Type,
    result_type: &Type,
    narms: Vec<N::MatchArm>,
    ref_mut: &Option<bool>,
) -> Vec<T::MatchArm> {
    narms
        .into_iter()
        .map(|narm| match_arm(context, subject_type, result_type, narm, ref_mut))
        .collect()
}

fn match_arm(
    context: &mut Context,
    subject_type: &Type,
    result_type: &Type,
    sp!(aloc, arm_): N::MatchArm,
    ref_mut: &Option<bool>,
) -> T::MatchArm {
    let N::MatchArm_ {
        pattern,
        binders,
        guard,
        guard_binders,
        rhs_binders,
        rhs,
    } = arm_;

    let bind_locs = binders.iter().map(|(_, sp!(loc, _))| *loc).collect();
    let msg = "Invalid type for pattern";
    let bind_vars = core::make_expr_list_tvars(context, pattern.loc, msg, bind_locs);

    let binders: Vec<(N::Var, Type)> = binders
        .into_iter()
        .zip(bind_vars)
        .map(|((mut_, x), ty)| {
            context.declare_local(mut_, x, ty.clone());
            (x, ty)
        })
        .collect();

    let ploc = pattern.loc;
    let pattern = match_pattern(context, pattern, ref_mut);

    subtype(
        context,
        ploc,
        || "Invalid pattern",
        pattern.ty.clone(),
        subject_type.clone(),
    );

    let binder_map: BTreeMap<N::Var, Type> = binders.clone().into_iter().collect();

    for (pat_var, guard_var) in guard_binders.clone() {
        use Type_::*;
        let ety = binder_map.get(&pat_var).unwrap().clone();
        let unfolded = core::unfold_type(&context.subst, ety.clone());
        let ty = match unfolded.value {
            Ref(false, inner) => sp(ety.loc, Ref(false, inner)),
            Ref(true, inner) => sp(ety.loc, Ref(false, inner)),
            _ => sp(ety.loc, Ref(false, Box::new(ety.clone()))),
        };
        context.declare_local(None, guard_var, ty);
    }

    let guard = guard.map(|guard| exp(context, guard));

    if let Some(guard) = &guard {
        let gloc = guard.exp.loc;
        subtype(
            context,
            gloc,
            || "Invalid guard condition",
            guard.ty.clone(),
            Type_::bool(gloc),
        );
    }

    let rhs = exp(context, rhs);
    subtype(
        context,
        rhs.exp.loc,
        || "Invalid right-hand side expression",
        rhs.ty.clone(),
        result_type.clone(),
    );

    sp(
        aloc,
        T::MatchArm_ {
            pattern,
            binders,
            guard,
            guard_binders,
            rhs_binders,
            rhs,
        },
    )
}

fn match_pattern(
    context: &mut Context,
    sp!(loc, pat_): N::MatchPattern,
    mut_ref: &Option<bool>, /* None -> value, Some(false) -> imm ref, Some(true) -> mut ref */
) -> T::MatchPattern {
    use N::MatchPattern_ as P;
    use T::UnannotatedPat_ as TP;

    macro_rules! rtype {
        ($ty:expr) => {
            if let Some(mut_) = mut_ref {
                sp($ty.loc, Type_::Ref(*mut_, Box::new($ty)))
            } else {
                $ty
            }
        };
    }

    match pat_ {
        P::Constructor(m, enum_, variant, tys_opt, fields) => {
            let (bt, targs) = core::make_enum_type(context, loc, &m, &enum_, tys_opt);
            let typed_fields = add_variant_field_types(
                context,
                loc,
                "pattern",
                &m,
                &enum_,
                &variant,
                targs.clone(),
                fields,
            );
            let tfields = typed_fields.map(|f, (idx, (fty, tpat))| {
                let tpat = match_pattern(context, tpat, mut_ref);
                let fty_ref = rtype!(fty);
                let fty_out = subtype(
                    context,
                    f.loc(),
                    || "Invalid pattern field type",
                    tpat.ty.clone(),
                    fty_ref,
                );
                (idx, (fty_out, tpat))
            });
            if !context.is_current_module(&m) {
                let msg = format!(
                    "Invalid deconstructing pattern for '{}::{}::{}'.\n All enums can only be \
                     matched in the module in which they are declared",
                    &m, &enum_, &variant
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::Visibility, (loc, msg)));
            }
            let bt = rtype!(bt);
            let pat_ = if mut_ref.is_some() {
                TP::BorrowConstructor(m, enum_, variant, targs, tfields)
            } else {
                TP::Constructor(m, enum_, variant, targs, tfields)
            };
            T::pat(bt, sp(loc, pat_))
        }
        P::Binder(x) => {
            let x_ty = context.get_local_type(&x);
            T::pat(x_ty, sp(loc, TP::Binder(x)))
        }
        P::Literal(v) => {
            let ty = match &v.value {
                Value_::InferredNum(_) => core::make_num_tvar(context, loc),
                _ => v.value.type_(loc).unwrap(),
            };
            context.add_ability_constraint(
                loc,
                Some(format!(
                    "Cannot ignore values without the '{}' ability. \
                    Literal patterns copy their values for equality checking",
                    Ability_::Copy
                )),
                ty.clone(),
                Ability_::Copy,
            );
            T::pat(rtype!(ty), sp(loc, TP::Literal(v)))
        }
        P::Wildcard => {
            let ty = core::make_tvar(context, loc);
            if mut_ref.is_none() {
                // If the thing we are matching isn't a ref, a wildcard drops it.
                context.add_ability_constraint(
                    loc,
                    Some(format!(
                        "Cannot ignore values without the '{}' ability. \
                        '_' patterns discard their values",
                        Ability_::Drop
                    )),
                    ty.clone(),
                    Ability_::Drop,
                );
            }
            T::pat(rtype!(ty), sp(loc, TP::Wildcard))
        }
        P::Or(lhs, rhs) => {
            let lpat = match_pattern(context, *lhs, mut_ref);
            let rpat = match_pattern(context, *rhs, mut_ref);
            let ty = join(
                context,
                loc,
                || -> String { panic!("ICE unresolved error join, failed") },
                lpat.ty.clone(),
                rpat.ty.clone(),
            );
            let pat = sp(loc, TP::Or(Box::new(lpat), Box::new(rpat)));
            T::pat(ty, pat)
        }
        P::At(x, inner) => {
            let inner = match_pattern(context, *inner, mut_ref);
            let x_ty = context.get_local_type(&x);
            let ty = subtype(
                context,
                inner.pat.loc,
                || "Invalid inner pattern type".to_string(),
                inner.ty.clone(),
                x_ty.clone(),
            );
            T::pat(ty, sp(loc, TP::At(x, Box::new(inner))))
        }
        P::ErrorPat => T::pat(core::make_tvar(context, loc), sp(loc, TP::ErrorPat)),
    }
}

//**************************************************************************************************
// Locals and LValues
//**************************************************************************************************

fn lvalues_expected_types(
    context: &mut Context,
    sp!(_loc, bs_): &T::LValueList,
) -> Vec<Option<N::Type>> {
    bs_.iter()
        .map(|b| lvalue_expected_types(context, b))
        .collect()
}

fn lvalue_expected_types(_context: &mut Context, sp!(loc, b_): &T::LValue) -> Option<N::Type> {
    use N::Type_::*;
    use T::LValue_ as L;
    let loc = *loc;
    match b_ {
        L::Ignore => None,
        L::Var { ty, .. } => Some(*ty.clone()),
        L::BorrowUnpack(mut_, m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(*m, *s));
            Some(sp(
                loc,
                Ref(*mut_, Box::new(sp(loc, Apply(None, tn, tys.clone())))),
            ))
        }
        L::Unpack(m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(*m, *s));
            Some(sp(loc, Apply(None, tn, tys.clone())))
        }
        L::BorrowUnpackVariant(..) | L::UnpackVariant(..) => {
            panic!("ICE shouldn't occur before match expansions")
        }
    }
}

#[derive(Clone, Copy)]
enum LValueCase {
    Bind,
    Assign,
}

fn bind_list(context: &mut Context, ls: N::LValueList, ty_opt: Option<Type>) -> T::LValueList {
    lvalue_list(context, LValueCase::Bind, ls, ty_opt)
}

fn assign_list(context: &mut Context, ls: N::LValueList, rvalue_ty: Type) -> T::LValueList {
    lvalue_list(context, LValueCase::Assign, ls, Some(rvalue_ty))
}

fn lvalue_list(
    context: &mut Context,
    case: LValueCase,
    sp!(loc, nlvalues): N::LValueList,
    ty_opt: Option<Type>,
) -> T::LValueList {
    use LValueCase as C;
    let arity = nlvalues.len();
    let locs = nlvalues.iter().map(|sp!(loc, _)| *loc).collect();
    let msg = "Invalid type for local";
    let ty_vars = core::make_expr_list_tvars(context, loc, msg, locs);
    let var_ty = match arity {
        0 => sp(loc, Type_::Unit),
        1 => sp(loc, ty_vars[0].value.clone()),
        _ => Type_::multiple(loc, ty_vars.clone()),
    };
    if let Some(ty) = ty_opt {
        let result = subtype_opt(
            context,
            loc,
            || {
                format!(
                    "Invalid value for {}",
                    match case {
                        C::Bind => "binding",
                        C::Assign => "assignment",
                    }
                )
            },
            ty,
            var_ty,
        );
        if result.is_none() {
            for ty_var in ty_vars.clone() {
                let ety = context.error_type(ty_var.loc);
                join(
                    context,
                    loc,
                    || -> String { panic!("ICE unresolved error join, failed") },
                    ty_var,
                    ety,
                );
            }
        }
    }
    assert!(ty_vars.len() == nlvalues.len(), "ICE invalid lvalue tvars");
    let tbinds = nlvalues
        .into_iter()
        .zip(ty_vars)
        .map(|(l, t)| lvalue(context, case, l, t))
        .collect();
    sp(loc, tbinds)
}

fn lvalue(
    context: &mut Context,
    case: LValueCase,
    sp!(loc, nl_): N::LValue,
    ty: Type,
) -> T::LValue {
    use LValueCase as C;

    use N::LValue_ as NL;
    use T::LValue_ as TL;
    let tl_ = match nl_ {
        NL::Ignore => {
            context.add_ability_constraint(
                loc,
                Some(format!(
                    "Cannot ignore values without the '{}' ability. The value must be used",
                    Ability_::Drop
                )),
                ty,
                Ability_::Drop,
            );
            TL::Ignore
        }
        NL::Var {
            mut_,
            var,
            unused_binding,
        } => {
            let var_ty = match case {
                C::Bind => {
                    context.declare_local(mut_, var, ty.clone());
                    ty
                }
                C::Assign => {
                    check_mutability(context, loc, "assignment", &var);
                    let var_ty = context.get_local_type(&var);
                    subtype(
                        context,
                        loc,
                        || format!("Invalid assignment to variable '{}'", &var.value.name),
                        ty,
                        var_ty.clone(),
                    );
                    var_ty
                }
            };
            TL::Var {
                var,
                ty: Box::new(var_ty),
                unused_binding,
            }
        }
        NL::Unpack(m, n, ty_args_opt, fields) => {
            let (bt, targs) = core::make_struct_type(context, loc, &m, &n, ty_args_opt);
            let (ref_mut, ty_inner) = match core::unfold_type(&context.subst, ty.clone()).value {
                Type_::Ref(mut_, inner) => (Some(mut_), *inner),
                _ => {
                    // Do not need base constraint because of the join below
                    (None, ty)
                }
            };
            match case {
                C::Bind => subtype(
                    context,
                    loc,
                    || "Invalid deconstruction binding",
                    bt,
                    ty_inner,
                ),
                C::Assign => subtype(
                    context,
                    loc,
                    || "Invalid deconstruction assignment",
                    bt,
                    ty_inner,
                ),
            };
            let verb = match case {
                C::Bind => "binding",
                C::Assign => "assignment",
            };
            let typed_fields =
                add_struct_field_types(context, loc, verb, &m, &n, targs.clone(), fields);
            let tfields = typed_fields.map(|f, (idx, (fty, nl))| {
                let nl_ty = match ref_mut {
                    None => fty.clone(),
                    Some(mut_) => sp(f.loc(), Type_::Ref(mut_, Box::new(fty.clone()))),
                };
                let tl = lvalue(context, case, nl, nl_ty);
                (idx, (fty, tl))
            });
            if !context.is_current_module(&m) {
                let msg = format!(
                    "Invalid deconstruction {} of '{}::{}'.\n All structs can only be \
                     deconstructed in the module in which they are declared",
                    verb, &m, &n,
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::Visibility, (loc, msg)));
            }
            match ref_mut {
                None => TL::Unpack(m, n, targs, tfields),
                Some(mut_) => TL::BorrowUnpack(mut_, m, n, targs, tfields),
            }
        }
    };
    sp(loc, tl_)
}

fn check_mutation(context: &mut Context, loc: Loc, given_ref: Type, rvalue_ty: &Type) -> Type {
    let inner = core::make_tvar(context, loc);
    let ref_ty = sp(loc, Type_::Ref(true, Box::new(inner.clone())));
    let res_ty = subtype(
        context,
        loc,
        || "Invalid mutation. Expected a mutable reference",
        given_ref,
        ref_ty,
    );
    subtype(
        context,
        loc,
        || "Invalid mutation. New value is not valid for the reference",
        rvalue_ty.clone(),
        inner.clone(),
    );
    context.add_ability_constraint(
        loc,
        Some(format!(
            "Invalid mutation. Mutation requires the '{}' ability as the old value is destroyed",
            Ability_::Drop
        )),
        inner,
        Ability_::Drop,
    );
    res_ty
}

fn check_mutability(context: &mut Context, eloc: Loc, usage: &str, v: &N::Var) {
    let (decl_loc, mut_) = context.mark_mutable_usage(eloc, v);
    if mut_.is_none() {
        let v = &v.value.name;
        let usage_msg = format!("Invalid {usage} of immutable variable '{v}'");
        let decl_msg =
            format!("To use the variable mutably, it must be declared 'mut', e.g. 'mut {v}'");
        if context.env.edition(context.current_package()) == Edition::E2024_MIGRATION {
            context
                .env
                .add_diag(diag!(Migration::NeedsLetMut, (decl_loc, decl_msg.clone()),))
        }
        context.env.add_diag(diag!(
            TypeSafety::InvalidImmVariableUsage,
            (eloc, usage_msg),
            (decl_loc, decl_msg),
        ))
    }
}

//**************************************************************************************************
// Fields
//**************************************************************************************************

fn resolve_field(context: &mut Context, loc: Loc, ty: Type, field: &Field) -> Type {
    use TypeName_::*;
    use Type_::*;
    const UNINFERRED_MSG: &str =
        "Could not infer the type before field access. Try annotating here";
    let msg = || format!("Unbound field '{}'", field);
    match core::ready_tvars(&context.subst, ty) {
        sp!(_, UnresolvedError) => context.error_type(loc),
        sp!(tloc, Anything) => {
            context.env.add_diag(diag!(
                TypeSafety::UninferredType,
                (loc, msg()),
                (tloc, UNINFERRED_MSG),
            ));
            context.error_type(loc)
        }
        sp!(tloc, Var(i)) if !context.subst.is_num_var(i) => {
            context.env.add_diag(diag!(
                TypeSafety::UninferredType,
                (loc, msg()),
                (tloc, UNINFERRED_MSG),
            ));
            context.error_type(loc)
        }
        sp!(_, Apply(_, sp!(_, ModuleType(m, n)), targs)) => {
            if !context.is_current_module(&m) {
                let msg = format!(
                    "Invalid access of field '{}' on '{}::{}'. Fields can only be accessed inside \
                     the struct's module",
                    field, &m, &n
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::Visibility, (loc, msg)));
            }
            match context.datatype_kind(&m, &n) {
                DatatypeKind::Struct => {
                    core::make_struct_field_type(context, loc, &m, &n, targs, field)
                }
                DatatypeKind::Enum => {
                    let msg = format!(
                        "Invalid access of field '{}' on '{}::{}'. Fields can only be accessed on \
                         structs, not enums",
                        field, &m, &n
                    );
                    context
                        .env
                        .add_diag(diag!(TypeSafety::ExpectedSpecificType, (loc, msg)));
                    context.error_type(loc)
                }
            }
        }
        t => {
            let smsg = format!(
                "Expected a struct type in the current module but got: {}",
                core::error_format(&t, &context.subst)
            );
            context.env.add_diag(diag!(
                TypeSafety::ExpectedSpecificType,
                (loc, msg()),
                (t.loc, smsg),
            ));
            context.error_type(loc)
        }
    }
}

fn add_struct_field_types<T>(
    context: &mut Context,
    loc: Loc,
    verb: &str,
    m: &ModuleIdent,
    n: &DatatypeName,
    targs: Vec<Type>,
    fields: Fields<T>,
) -> Fields<(Type, T)> {
    let maybe_fields_ty = core::make_struct_field_types(context, loc, m, n, targs);
    let mut fields_ty = match maybe_fields_ty {
        N::StructFields::Defined(m) => m,
        N::StructFields::Native(nloc) => {
            let msg = format!(
                "Invalid {} usage for native struct '{}::{}'. Native structs cannot be directly \
                 constructed/deconstructed, and their fields cannot be dirctly accessed",
                verb, m, n
            );
            context.env.add_diag(diag!(
                TypeSafety::InvalidNativeUsage,
                (loc, msg),
                (nloc, "Struct declared 'native' here")
            ));
            return fields.map(|f, (idx, x)| (idx, (context.error_type(f.loc()), x)));
        }
    };
    for (_, f_, _) in &fields_ty {
        if fields.get_(f_).is_none() {
            let msg = format!("Missing {} for field '{}' in '{}::{}'", verb, f_, m, n);
            context
                .env
                .add_diag(diag!(TypeSafety::TooFewArguments, (loc, msg)))
        }
    }
    fields.map(|f, (idx, x)| {
        let fty = match fields_ty.remove(&f) {
            None => {
                context.env.add_diag(diag!(
                    NameResolution::UnboundField,
                    (loc, format!("Unbound field '{}' in '{}::{}'", &f, m, n))
                ));
                context.error_type(f.loc())
            }
            Some((_, fty)) => fty,
        };
        (idx, (fty, x))
    })
}

fn add_variant_field_types<T>(
    context: &mut Context,
    loc: Loc,
    verb: &str,
    m: &ModuleIdent,
    n: &DatatypeName,
    v: &VariantName,
    targs: Vec<Type>,
    fields: Fields<T>,
) -> Fields<(Type, T)> {
    let maybe_fields_ty = core::make_variant_field_types(context, loc, m, n, v, targs);
    let mut fields_ty = match maybe_fields_ty {
        N::VariantFields::Defined(m) => m,
        N::VariantFields::Empty => {
            if !fields.is_empty() {
                let msg = format!(
                    "Invalid usage for empty variant '{}::{}::{}'. Empty variants do not take \
                     any arguments.",
                    m, n, v
                );
                context
                    .env
                    .add_diag(diag!(TypeSafety::InvalidNativeUsage, (loc, msg),));
                return fields.map(|f, (idx, x)| (idx, (context.error_type(f.loc()), x)));
            } else {
                return Fields::new();
            }
        }
    };
    for (_, f_, _) in &fields_ty {
        if fields.get_(f_).is_none() {
            let msg = format!(
                "Missing {} for field '{}' in '{}::{}::{}'",
                verb, f_, m, n, v
            );
            context
                .env
                .add_diag(diag!(TypeSafety::TooFewArguments, (loc, msg)))
        }
    }
    fields.map(|f, (idx, x)| {
        let fty = match fields_ty.remove(&f) {
            None => {
                context.env.add_diag(diag!(
                    NameResolution::UnboundField,
                    (
                        loc,
                        format!("Unbound field '{}' in '{}::{}::{}'", &f, m, n, v)
                    )
                ));
                context.error_type(f.loc())
            }
            Some((_, fty)) => fty,
        };
        (idx, (fty, x))
    })
}

enum ExpDotted_ {
    Exp(Box<T::Exp>),
    TmpBorrow(Box<T::Exp>, Box<Type>),
    Dot(Box<ExpDotted>, Field, Box<Type>),
}
type ExpDotted = Spanned<ExpDotted_>;

// if constraint_verb is None, no single typeconstraint is applied
fn exp_dotted(
    context: &mut Context,
    constraint_verb: Option<&str>,
    sp!(dloc, ndot_): N::ExpDotted,
) -> (ExpDotted, Type) {
    use N::ExpDotted_ as NE;
    let (edot_, ty) = match ndot_ {
        NE::Exp(ne) => {
            use Type_::*;
            let e = exp(context, ne);
            warn_on_constant_borrow(context, dloc, &e);
            let ety = &e.ty;
            let unfolded = core::unfold_type(&context.subst, ety.clone());
            let (borrow_needed, ty) = match unfolded.value {
                Ref(_, inner) => (false, *inner),
                _ => (true, ety.clone()),
            };
            let edot_ = if borrow_needed {
                if let Some(verb) = constraint_verb {
                    context.add_single_type_constraint(
                        dloc,
                        format!("Invalid {}", verb),
                        ty.clone(),
                    );
                }
                ExpDotted_::TmpBorrow(e, Box::new(ty.clone()))
            } else {
                ExpDotted_::Exp(e)
            };
            (edot_, ty)
        }
        NE::Dot(nlhs, field) => {
            let (lhs, inner) = exp_dotted(context, Some("dot access"), *nlhs);
            let field_ty = resolve_field(context, dloc, inner, &field);
            (
                ExpDotted_::Dot(Box::new(lhs), field, Box::new(field_ty.clone())),
                field_ty,
            )
        }
    };
    (sp(dloc, edot_), ty)
}

fn exp_dotted_to_borrow(
    context: &mut Context,
    loc: Loc,
    mut_: bool,
    sp!(dloc, dot_): ExpDotted,
) -> T::Exp {
    use Type_::*;
    use T::UnannotatedExp_ as TE;
    match dot_ {
        ExpDotted_::Exp(e) => *e,
        ExpDotted_::TmpBorrow(eb, desired_inner_ty) => {
            let eb_ty = eb.ty;
            let sp!(ebloc, eb_) = eb.exp;
            let e_ = match eb_ {
                TE::Use(v) => {
                    if mut_ {
                        check_mutability(context, loc, "mutable borrow", &v);
                    }
                    TE::BorrowLocal(mut_, v)
                }
                eb_ => {
                    match &eb_ {
                        TE::Move { from_user, .. } | TE::Copy { from_user, .. } => {
                            assert!(*from_user)
                        }
                        _ => (),
                    }
                    TE::TempBorrow(mut_, Box::new(T::exp(eb_ty, sp(ebloc, eb_))))
                }
            };
            let ty = sp(loc, Ref(mut_, desired_inner_ty));
            T::exp(ty, sp(dloc, e_))
        }
        ExpDotted_::Dot(lhs, field, field_ty) => {
            let lhs_borrow = exp_dotted_to_borrow(context, dloc, mut_, *lhs);
            let sp!(tyloc, unfolded_) = core::unfold_type(&context.subst, lhs_borrow.ty.clone());
            let lhs_mut = match unfolded_ {
                Ref(lhs_mut, _) => lhs_mut,
                _ => panic!(
                    "ICE expected a ref from exp_dotted borrow, otherwise should have gotten a \
                     TmpBorrow"
                ),
            };
            // lhs is immutable and current borrow is mutable
            if !lhs_mut && mut_ {
                context.env.add_diag(diag!(
                    ReferenceSafety::RefTrans,
                    (loc, "Invalid mutable borrow from an immutable reference"),
                    (tyloc, "Immutable because of this position"),
                ))
            }
            let e_ = TE::Borrow(mut_, Box::new(lhs_borrow), field);
            let ty = sp(loc, Ref(mut_, field_ty));
            T::exp(ty, sp(dloc, e_))
        }
    }
}

fn exp_dotted_to_owned_value(
    context: &mut Context,
    usage: DottedUsage,
    eloc: Loc,
    edot: ExpDotted,
    inner_ty: Type,
) -> T::Exp {
    use T::UnannotatedExp_ as TE;
    match edot {
        sp!(_, ExpDotted_::Exp(lhs)) | sp!(_, ExpDotted_::TmpBorrow(lhs, _)) => {
            debug_assert!(
                usage == DottedUsage::Use,
                "ICE this case should only come from method calls. \
                move/copy/borrow should be covered above"
            );
            *lhs
        }
        edot => {
            let name = match &edot {
                sp!(_, ExpDotted_::Exp(_)) | sp!(_, ExpDotted_::TmpBorrow(_, _)) => {
                    panic!("ICE covered above")
                }
                sp!(_, ExpDotted_::Dot(_, name, _)) => *name,
            };
            let eborrow = exp_dotted_to_borrow(context, eloc, false, edot);
            let case = match usage {
                DottedUsage::Move(loc) => {
                    let new_syntax = context.env.check_feature(
                        FeatureGate::Move2024Paths,
                        context.current_package(),
                        loc,
                    );
                    if new_syntax {
                        let msg = "Invalid 'move'. 'move' works only with \
                            variables, e.g. 'move x'. 'move' on a path access is not supported";
                        context
                            .env
                            .add_diag(diag!(TypeSafety::InvalidMoveOp, (loc, msg)));
                    }
                    None
                }
                DottedUsage::Copy(loc) => {
                    context.env.check_feature(
                        FeatureGate::Move2024Paths,
                        context.current_package(),
                        loc,
                    );
                    Some("'copy'")
                }
                DottedUsage::Use => Some("implicit copy"),
                DottedUsage::Borrow(_) => unreachable!("ICE covered above"),
            };
            if let Some(case) = case {
                context.add_ability_constraint(
                    eloc,
                    Some(format!(
                        "Invalid {} of field '{}' without the '{}' ability",
                        case,
                        name,
                        Ability_::COPY,
                    )),
                    inner_ty.clone(),
                    Ability_::Copy,
                );
                T::exp(inner_ty, sp(eloc, TE::Dereference(Box::new(eborrow))))
            } else {
                // 'move' case, which is not supported
                T::exp(context.error_type(eloc), sp(eloc, TE::UnresolvedError))
            }
        }
    }
}

fn warn_on_constant_borrow(context: &mut Context, loc: Loc, e: &T::Exp) {
    use T::UnannotatedExp_ as TE;
    if matches!(&e.exp.value, TE::Constant(_, _)) {
        let msg = "This access will make a new copy of the constant. Consider binding the value to a variable first to make this copy explicit";
        context
            .env
            .add_diag(diag!(TypeSafety::ImplicitConstantCopy, (loc, msg)))
    }
}

impl crate::shared::ast_debug::AstDebug for ExpDotted_ {
    fn ast_debug(&self, w: &mut crate::shared::ast_debug::AstWriter) {
        use ExpDotted_ as D;
        match self {
            D::Exp(e) => e.ast_debug(w),
            D::TmpBorrow(e, ty) => {
                w.write("&tmp ");
                w.annotate(|w| e.ast_debug(w), ty)
            }
            D::Dot(e, n, ty) => {
                e.ast_debug(w);
                w.write(".");
                w.annotate(|w| w.write(&format!("{}", n)), ty)
            }
        }
    }
}

//**************************************************************************************************
// Calls
//**************************************************************************************************

fn method_call(
    context: &mut Context,
    loc: Loc,
    edotted: ExpDotted,
    edotted_ty: Type,
    method: Name,
    ty_args_opt: Option<Vec<Type>>,
    argloc: Loc,
    mut args: Vec<T::Exp>,
) -> Option<(Type, T::UnannotatedExp_)> {
    use T::UnannotatedExp_ as TE;
    let (m, f, fty, first_arg) =
        method_call_resolve(context, loc, edotted, edotted_ty, method, ty_args_opt)?;
    args.insert(0, first_arg);
    let (mut call, ret_ty) = module_call_impl(context, loc, m, f, fty, argloc, args);
    call.method_name = Some(method);
    Some((ret_ty, TE::ModuleCall(Box::new(call))))
}

fn method_call_resolve(
    context: &mut Context,
    loc: Loc,
    mut edotted: ExpDotted,
    edotted_ty: Type,
    method: Name,
    ty_args_opt: Option<Vec<Type>>,
) -> Option<(ModuleIdent, FunctionName, ResolvedFunctionType, T::Exp)> {
    use TypeName_ as TN;
    use Type_ as Ty;
    let edotted_ty_unfolded = core::unfold_type(&context.subst, edotted_ty.clone());
    let edotted_bty = edotted_ty_base(&edotted_ty_unfolded);
    let tn = match &edotted_bty.value {
        Ty::Apply(_, tn @ sp!(_, TN::ModuleType(_, _) | TN::Builtin(_)), _) => tn,
        t => {
            let msg = match t {
                Ty::Anything => {
                    "Unable to infer type for method call. Try annotating this type".to_owned()
                }
                Ty::Unit | Ty::Apply(_, sp!(_, TN::Multiple(_)), _) | Ty::Fun(_, _) => {
                    let tsubst = core::error_format_(t, &context.subst);
                    format!(
                        "Method calls are only supported on single types. \
                          Got an expression of type: {tsubst}",
                    )
                }
                Ty::Param(_) => {
                    let tsubst = core::error_format_(t, &context.subst);
                    format!(
                        "Method calls are not supported on type parameters. \
                        Got an expression of type: {tsubst}",
                    )
                }
                Ty::UnresolvedError => {
                    assert!(context.env.has_errors());
                    return None;
                }
                Ty::Ref(_, _) | Ty::Var(_) => panic!("ICE unfolding failed"),
                Ty::Apply(_, _, _) => unreachable!(),
            };
            context.env.add_diag(diag!(
                TypeSafety::InvalidMethodCall,
                (loc, "Invalid method call"),
                (edotted_ty.loc, msg),
            ));
            return None;
        }
    };
    let (m, f, fty) =
        core::make_method_call_type(context, loc, &edotted_ty, tn, method, ty_args_opt)?;

    let first_arg = match &fty.params[0].1.value {
        Ty::Ref(mut_, _) => {
            // add a borrow if needed
            let mut cur = &mut edotted;
            loop {
                match cur {
                    sp!(loc, ExpDotted_::Exp(e)) => {
                        let e_ty = e.ty.clone();
                        match core::unfold_type(&context.subst, e_ty.clone()).value {
                            Ty::Ref(_, _) => (),
                            _ => *cur = sp(*loc, ExpDotted_::TmpBorrow(e.clone(), Box::new(e_ty))),
                        };
                        break;
                    }
                    sp!(_, ExpDotted_::TmpBorrow(_, _)) => break,
                    sp!(_, ExpDotted_::Dot(l, _, _)) => cur = l,
                };
            }
            exp_dotted_to_borrow(context, loc, *mut_, edotted)
        }
        _ => exp_dotted_to_owned_value(context, DottedUsage::Use, loc, edotted, edotted_ty),
    };
    Some((m, f, fty, first_arg))
}

fn edotted_ty_base(ty: &Type) -> &Type {
    match &ty.value {
        Type_::Unit
        | Type_::Param(_)
        | Type_::Anything
        | Type_::UnresolvedError
        | Type_::Apply(_, _, _)
        | Type_::Fun(_, _) => ty,
        Type_::Ref(_, inner) => inner,
        Type_::Var(_) => panic!("ICE unfolding failed"),
    }
}

fn module_call(
    context: &mut Context,
    loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    ty_args_opt: Option<Vec<Type>>,
    argloc: Loc,
    args: Vec<T::Exp>,
) -> (Type, T::UnannotatedExp_) {
    let fty = core::make_function_type(context, loc, &m, &f, ty_args_opt);
    let (call, ret_ty) = module_call_impl(context, loc, m, f, fty, argloc, args);
    (ret_ty, T::UnannotatedExp_::ModuleCall(Box::new(call)))
}

fn module_call_impl(
    context: &mut Context,
    loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    fty: ResolvedFunctionType,
    argloc: Loc,
    args: Vec<T::Exp>,
) -> (T::ModuleCall, Type) {
    let ResolvedFunctionType {
        declared,
        macro_,
        ty_args,
        params: parameters,
        return_,
    } = fty;
    check_call_target(
        context, loc, /* is_macro_call */ None, macro_, declared, f,
    );
    let (arguments, arg_tys) = call_args(
        context,
        loc,
        || format!("Invalid call of '{}::{}'", &m, &f),
        parameters.len(),
        argloc,
        args,
    );
    assert!(arg_tys.len() == parameters.len());
    for (arg_ty, (param, param_ty)) in arg_tys.into_iter().zip(parameters.clone()) {
        let msg = || {
            format!(
                "Invalid call of '{}::{}'. Invalid argument for parameter '{}'",
                &m, &f, &param.value.name
            )
        };
        subtype(context, loc, msg, arg_ty, param_ty);
    }
    let params_ty_list = parameters.into_iter().map(|(_, ty)| ty).collect();
    let call = T::ModuleCall {
        module: m,
        name: f,
        type_arguments: ty_args,
        arguments,
        parameter_types: params_ty_list,
        method_name: None,
    };
    context
        .used_module_members
        .entry(m.value)
        .or_default()
        .insert(f.value());
    (call, return_)
}

fn builtin_call(
    context: &mut Context,
    loc: Loc,
    sp!(bloc, nb_): N::BuiltinFunction,
    argloc: Loc,
    args: Vec<T::Exp>,
) -> (Type, T::UnannotatedExp_) {
    use N::BuiltinFunction_ as NB;
    use T::BuiltinFunction_ as TB;
    let mut mk_ty_arg = |ty_arg_opt| match ty_arg_opt {
        None => core::make_tvar(context, loc),
        Some(ty_arg) => core::instantiate(context, ty_arg),
    };
    let (b_, params_ty, ret_ty);
    match nb_ {
        NB::Freeze(ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::Freeze(ty_arg.clone());
            params_ty = vec![sp(bloc, Type_::Ref(true, Box::new(ty_arg.clone())))];
            ret_ty = sp(loc, Type_::Ref(false, Box::new(ty_arg)));
        }
        NB::Assert(is_macro) => {
            b_ = TB::Assert(is_macro);
            params_ty = vec![Type_::bool(bloc), Type_::u64(bloc)];
            ret_ty = sp(loc, Type_::Unit);
        }
    };
    let (arguments, arg_tys) = call_args(
        context,
        loc,
        || format!("Invalid call of '{}'", &b_),
        params_ty.len(),
        argloc,
        args,
    );
    assert!(arg_tys.len() == params_ty.len());
    for ((idx, arg_ty), param_ty) in arg_tys.into_iter().enumerate().zip(params_ty) {
        let msg = || {
            format!(
                "Invalid call of '{}'. Invalid argument for parameter '{}'",
                &b_, idx
            )
        };
        subtype(context, loc, msg, arg_ty, param_ty);
    }
    let call = T::UnannotatedExp_::Builtin(Box::new(sp(bloc, b_)), arguments);
    (ret_ty, call)
}

fn vector_pack(
    context: &mut Context,
    eloc: Loc,
    vec_loc: Loc,
    ty_arg_opt: Option<Type>,
    argloc: Loc,
    args_: Vec<T::Exp>,
) -> (Type, T::UnannotatedExp_) {
    let arity = args_.len();
    let (eargs, args_ty) = call_args(
        context,
        eloc,
        || -> String { panic!("ICE. could not create vector args") },
        arity,
        argloc,
        args_,
    );
    let mut inferred_vec_ty_arg = core::make_tvar(context, eloc);
    for arg_ty in args_ty {
        // TODO this could be improved... A LOT
        // this ends up generating a new tvar chain for each element in the vector
        // which ends up being n^2 chains
        inferred_vec_ty_arg = join(
            context,
            eloc,
            || "Invalid 'vector' instantiation. Incompatible argument",
            inferred_vec_ty_arg,
            arg_ty,
        );
    }
    let vec_ty_arg = match ty_arg_opt {
        None => inferred_vec_ty_arg,
        Some(ty_arg) => {
            let ty_arg = core::instantiate(context, ty_arg);
            subtype(
                context,
                eloc,
                || "Invalid 'vector' instantiation. Invalid argument type",
                inferred_vec_ty_arg,
                ty_arg.clone(),
            );
            ty_arg
        }
    };
    context.add_base_type_constraint(eloc, "Invalid 'vector' type", vec_ty_arg.clone());
    let ty_vec = Type_::vector(eloc, vec_ty_arg.clone());
    let e_ = T::UnannotatedExp_::Vector(vec_loc, arity, Box::new(vec_ty_arg), eargs);
    (ty_vec, e_)
}

fn call_args<S: std::fmt::Display, F: Fn() -> S>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    arity: usize,
    argloc: Loc,
    mut args: Vec<T::Exp>,
) -> (Box<T::Exp>, Vec<Type>) {
    use T::UnannotatedExp_ as TE;
    let tys = args.iter().map(|e| e.ty.clone()).collect();
    let tys = make_arg_types(context, loc, msg, arity, argloc, tys);
    let arg = match args.len() {
        0 => T::exp(
            sp(argloc, Type_::Unit),
            sp(argloc, TE::Unit { trailing: false }),
        ),
        1 => args.pop().unwrap(),
        _ => {
            let ty = Type_::multiple(argloc, tys.clone());
            let items = args.into_iter().map(T::single_item).collect();
            T::exp(ty, sp(argloc, TE::ExpList(items)))
        }
    };
    (Box::new(arg), tys)
}

fn make_arg_types<S: std::fmt::Display, F: Fn() -> S>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    arity: usize,
    argloc: Loc,
    mut given: Vec<Type>,
) -> Vec<Type> {
    let given_len = given.len();
    core::check_call_arity(context, loc, msg, arity, argloc, given_len);
    while given.len() < arity {
        given.push(context.error_type(argloc))
    }
    while given.len() > arity {
        given.pop();
    }
    given
}

fn check_call_target(
    context: &mut Context,
    call_loc: Loc,
    is_macro_call: Option<Loc>,
    declared_macro_modifier: Option<Loc>,
    declared: Loc,
    f: FunctionName,
) {
    let decl_is_macro = declared_macro_modifier.is_some();
    if is_macro_call.is_some() == decl_is_macro {
        return;
    }

    let macro_call_loc = is_macro_call.unwrap_or(call_loc);
    let decl_loc = declared_macro_modifier.unwrap_or(declared);
    let call_msg = if decl_is_macro {
        format!(
            "'{f}' is a macro function and must be called with a `!`. \
            Try replacing with '{f}!'"
        )
    } else {
        format!(
            "'{f}' is not a macro function and cannot be called with a `!`. \
            Try replacing with '{f}'"
        )
    };
    let decl_msg = if decl_is_macro {
        "'macro' function is declared here"
    } else {
        "Normal (non-'macro') function is declared here"
    };
    context.env.add_diag(diag!(
        TypeSafety::InvalidCallTarget,
        (macro_call_loc, call_msg),
        (decl_loc, decl_msg),
    ));
}

//**************************************************************************************************
// Macro
//**************************************************************************************************

fn macro_method_call(
    context: &mut Context,
    loc: Loc,
    edotted: ExpDotted,
    edotted_ty: Type,
    method: Name,
    macro_call_loc: Loc,
    ty_args_opt: Option<Vec<Type>>,
    argloc: Loc,
    nargs: Vec<N::Exp>,
) -> Option<(Type, T::UnannotatedExp_)> {
    let (m, f, fty, first_arg) =
        method_call_resolve(context, loc, edotted, edotted_ty, method, ty_args_opt)?;
    let mut args = vec![macro_expand::EvalStrategy::ByValue(first_arg)];
    args.extend(
        nargs
            .into_iter()
            .map(|e| macro_expand::EvalStrategy::ByName(convert_macro_arg_to_block(context, e))),
    );
    let (type_arguments, args, return_ty) =
        macro_call_impl(context, loc, m, f, macro_call_loc, fty, argloc, args);
    Some(expand_macro(
        context,
        loc,
        m,
        f,
        type_arguments,
        args,
        return_ty,
    ))
}

fn macro_module_call(
    context: &mut Context,
    loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    macro_call_loc: Loc,
    ty_args_opt: Option<Vec<Type>>,
    argloc: Loc,
    nargs: Vec<N::Exp>,
) -> (Type, T::UnannotatedExp_) {
    let fty = core::make_function_type(context, loc, &m, &f, ty_args_opt);
    let args = nargs
        .into_iter()
        .map(|e| macro_expand::EvalStrategy::ByName(convert_macro_arg_to_block(context, e)))
        .collect();
    let (type_arguments, args, return_ty) =
        macro_call_impl(context, loc, m, f, macro_call_loc, fty, argloc, args);
    expand_macro(context, loc, m, f, type_arguments, args, return_ty)
}

fn macro_call_impl(
    context: &mut Context,
    loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    macro_call_loc: Loc,
    fty: ResolvedFunctionType,
    argloc: Loc,
    mut args: Vec<macro_expand::EvalStrategy<T::Exp, N::Exp>>,
) -> (Vec<Type>, Vec<macro_expand::Arg>, Type) {
    use macro_expand::EvalStrategy;
    let ResolvedFunctionType {
        declared,
        macro_,
        ty_args,
        params: parameters,
        return_,
    } = fty;
    check_call_target(
        context,
        loc,
        /* is_macro_call */ Some(macro_call_loc),
        macro_,
        declared,
        f,
    );
    core::check_call_arity(
        context,
        loc,
        || format!("Invalid call of '{}::{}'", &m, &f),
        parameters.len(),
        argloc,
        args.len(),
    );
    // instantiate the param types to check for constraints, even if the argument isn't used
    for (_, param_ty) in &parameters {
        core::instantiate(context, param_ty.clone());
    }
    while args.len() < parameters.len() {
        args.push(EvalStrategy::ByName(sp(loc, N::Exp_::UnresolvedError)));
    }
    while args.len() > parameters.len() {
        args.pop();
    }
    assert!(args.len() == parameters.len());
    let args_with_ty = args
        .into_iter()
        .zip(parameters)
        .map(|(arg, (param, param_ty))| match arg {
            EvalStrategy::ByValue(e) => {
                let msg = || {
                    format!(
                        "Invalid call of '{}::{}'. Invalid argument for parameter '{}'",
                        &m, &f, &param.value.name
                    )
                };
                subtype(context, loc, msg, e.ty.clone(), param_ty.clone());
                EvalStrategy::ByValue(e)
            }
            EvalStrategy::ByName(ne) => {
                let expected_ty =
                    expected_by_name_arg_type(context, loc, &m, &f, &param, &ne, param_ty.clone());
                EvalStrategy::ByName((ne, expected_ty))
            }
        })
        .collect();
    context
        .used_module_members
        .entry(m.value)
        .or_default()
        .insert(f.value());
    (ty_args, args_with_ty, return_)
}

// If the argument is a lambda, we need to check that the lambda's type matches the expected type
// so that any calls to the lambda can be properly expanded
// Otherwise, we just return the parameters type
fn expected_by_name_arg_type(
    context: &mut Context,
    call_loc: Loc,
    m: &ModuleIdent,
    f: &FunctionName,
    param: &N::Var,
    ne: &N::Exp,
    param_ty: Type,
) -> Type {
    let (eloc, lambda) = match ne {
        sp!(eloc, N::Exp_::Lambda(l)) => (*eloc, l),
        _ => return param_ty,
    };
    let param_tys = lambda
        .parameters
        .value
        .iter()
        .map(|(p, ty_opt)| {
            if let Some(ty) = ty_opt {
                core::instantiate(context, ty.clone())
            } else {
                core::make_tvar(context, p.loc)
            }
        })
        .collect();
    let ret_ty = if let Some(ty) = lambda.return_type.clone() {
        core::instantiate(context, ty)
    } else {
        make_tvar(context, lambda.body.loc)
    };
    let tfun = sp(eloc, Type_::Fun(param_tys, Box::new(ret_ty)));
    let msg = || {
        format!(
            "Invalid call of '{}::{}'. Invalid argument for parameter '{}'",
            m, &f, &param.value.name
        )
    };
    subtype(context, call_loc, msg, tfun.clone(), param_ty);
    // prefer the lambda type over the parameters to preserve annotations on the lambda
    tfun
}

fn expand_macro(
    context: &mut core::Context,
    call_loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    type_args: Vec<N::Type>,
    args: Vec<macro_expand::Arg>,
    return_ty: Type,
) -> (Type, T::UnannotatedExp_) {
    use T::SequenceItem_ as TS;
    use T::UnannotatedExp_ as TE;

    let valid = context.add_macro_expansion(m, f, call_loc);
    if !valid {
        assert!(context.env.has_errors());
        return (context.error_type(call_loc), TE::UnresolvedError);
    }
    let res = match macro_expand::call(context, call_loc, m, f, type_args, args, return_ty) {
        None => {
            assert!(context.env.has_errors());
            (context.error_type(call_loc), TE::UnresolvedError)
        }
        Some(macro_expand::ExpandedMacro {
            by_value_args,
            body,
        }) => {
            // bind the locals
            let mut seq: VecDeque<_> = by_value_args
                .into_iter()
                .map(|(sp!(vloc, v_), e)| {
                    let lvalue_ = match v_ {
                        Some(var_) => N::LValue_::Var {
                            mut_: None,
                            var: sp(vloc, var_),
                            unused_binding: false,
                        },
                        None => N::LValue_::Ignore,
                    };
                    let lvalue = sp(vloc, lvalue_);
                    let lvalues = sp(vloc, vec![lvalue]);
                    let b = bind_list(context, lvalues, Some(e.ty.clone()));
                    let lvalue_ty = lvalues_expected_types(context, &b);
                    sp(b.loc, TS::Bind(b, lvalue_ty, Box::new(e)))
                })
                .collect();
            // add the body
            let body = exp(context, body);
            let ty = body.ty.clone();
            seq.push_back(sp(body.exp.loc, TS::Seq(body)));
            let use_funs = N::UseFuns::new(context.current_call_color());
            let e_ = TE::Block((use_funs, seq));
            (ty, e_)
        }
    };
    if context.pop_macro_expansion(call_loc, &m, &f) {
        res
    } else {
        (context.error_type(call_loc), TE::UnresolvedError)
    }
}

/// We need to make sure that arguments to macro calls are either lambdas or a Block
/// These arguments are call-by-name so the whole expression is substituted in. So we need to track
/// metadata about the scope where these expressions were originally written.
/// The Block lets us track two pieces of metadata
/// 1) We can track the use_fun_scope, which is used for resolving method calls correctly
/// 2) After substitution, we can mark the Block as coming from a macro expansion which is used
///    for tracking recursive macro calls
fn convert_macro_arg_to_block(context: &Context, sp!(loc, ne_): N::Exp) -> N::Exp {
    let ne_ = match ne_ {
        N::Exp_::Block(_) | N::Exp_::Lambda(_) | N::Exp_::UnresolvedError => ne_,
        ne_ => {
            let color = context.current_call_color();
            let seq_ = VecDeque::from([sp(loc, N::SequenceItem_::Seq(Box::new(sp(loc, ne_))))]);
            let seq = (N::UseFuns::new(color), seq_);
            let block = N::Block {
                name: None,
                from_macro_argument: None,
                seq,
            };
            N::Exp_::Block(block)
        }
    };
    sp(loc, ne_)
}

//**************************************************************************************************
// Utils
//**************************************************************************************************

fn process_attributes<T: TName>(context: &mut Context, all_attributes: &UniqueMap<T, Attribute>) {
    for (_, _, attr) in all_attributes {
        match &attr.value {
            Attribute_::Name(_) => (),
            Attribute_::Parameterized(_, attrs) => process_attributes(context, attrs),
            Attribute_::Assigned(_, val) => {
                let AttributeValue_::ModuleAccess(mod_access) = &val.value else {
                    continue;
                };
                if let ModuleAccess_::ModuleAccess(mident, name) = mod_access.value {
                    // conservatively assume that each `ModuleAccess` refers to a constant name
                    context
                        .used_module_members
                        .entry(mident.value)
                        .or_default()
                        .insert(name.value);
                }
            }
        }
    }
}

//**************************************************************************************************
// Follow-up warnings
//**************************************************************************************************

/// Generates warnings for unused mut declerations
/// Should be called at the end of functions/constants
fn unused_let_muts(context: &mut Context) {
    let locals = context.take_locals();
    let supports_let_mut = context
        .env
        .supports_feature(context.current_package, FeatureGate::LetMut);
    if !supports_let_mut {
        return;
    }
    for (v, local) in locals {
        let Local { mut_, used_mut, .. } = local;
        let Some(mut_loc) = mut_ else { continue };
        if used_mut.is_none() && !v.value.starts_with_underscore() {
            let decl_msg = format!("The variable '{}' is never used mutably", v.value.name);
            let mut_msg = "Consider removing the 'mut' declaration here";
            context.env.add_diag(diag!(
                UnusedItem::MutModifier,
                (v.loc, decl_msg),
                (mut_loc, mut_msg)
            ))
        }
    }
}

/// Generates warnings for unused (private) functions and unused constants.
/// Should be called after the whole program has been processed.
fn unused_module_members(context: &mut Context, mident: &ModuleIdent_, mdef: &T::ModuleDefinition) {
    if !mdef.is_source_module {
        // generate warnings only for modules compiled in this pass rather than for all modules
        // including pre-compiled libraries for which we do not have source code available and
        // cannot be analyzed in this pass
        return;
    }

    let is_sui_mode = context.env.package_config(mdef.package_name).flavor == Flavor::Sui;
    context
        .env
        .add_warning_filter_scope(mdef.warning_filter.clone());

    for (loc, name, c) in &mdef.constants {
        context
            .env
            .add_warning_filter_scope(c.warning_filter.clone());

        let members = context.used_module_members.get(mident);
        if members.is_none() || !members.unwrap().contains(name) {
            let msg = format!("The constant '{name}' is never used. Consider removing it.");
            context
                .env
                .add_diag(diag!(UnusedItem::Constant, (loc, msg)))
        }

        context.env.pop_warning_filter_scope();
    }

    for (loc, name, fun) in &mdef.functions {
        if fun.attributes.contains_key_(&TestingAttribute::Test.into()) {
            // functions with #[test] attribute are implicitly used
            continue;
        }
        if is_sui_mode && *name == sui_mode::INIT_FUNCTION_NAME {
            // a Sui-specific filter to avoid signaling that the init function is unused
            continue;
        }
        context
            .env
            .add_warning_filter_scope(fun.warning_filter.clone());

        let members = context.used_module_members.get(mident);
        if fun.entry.is_none()
            && matches!(fun.visibility, Visibility::Internal)
            && (members.is_none() || !members.unwrap().contains(name))
        {
            // TODO: postponing handling of friend functions until we decide what to do with them
            // vis-a-vis ideas around package-private
            let msg = format!(
                "The non-'public', non-'entry' function '{name}' is never called. \
                Consider removing it."
            );
            context
                .env
                .add_diag(diag!(UnusedItem::Function, (loc, msg)))
        }
        context.env.pop_warning_filter_scope();
    }

    context.env.pop_warning_filter_scope();
}
