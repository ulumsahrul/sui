// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::*;
use sui_package_resolver::{EnumDef, VariantDef};

use crate::error::Error;

use super::{
    move_module::MoveModule,
    move_struct::{MoveField, MoveStructTypeParameter},
    open_move_type::{abilities, MoveAbility},
    sui_address::SuiAddress,
};

pub(crate) struct MoveEnum {
    defining_id: SuiAddress,
    module: String,
    name: String,
    abilities: Vec<MoveAbility>,
    type_parameters: Vec<MoveStructTypeParameter>,
    variants: Vec<MoveEnumVariant>,
    checkpoint_viewed_at: u64,
}

/// Information for a particular Move variant
pub(crate) struct MoveEnumVariant {
    name: String,
    fields: Vec<MoveField>,
}

/// Description of an enum type, defined in a Move module.
#[Object]
impl MoveEnum {
    /// The module this enum was originally defined in.
    async fn module(&self, ctx: &Context<'_>) -> Result<MoveModule> {
        let Some(module) = MoveModule::query(
            ctx.data_unchecked(),
            self.defining_id,
            &self.module,
            self.checkpoint_viewed_at,
        )
        .await
        .extend()?
        else {
            return Err(Error::Internal(format!(
                "Failed to load module for enum: {}::{}::{}",
                self.defining_id, self.module, self.name,
            )))
            .extend();
        };

        Ok(module)
    }

    /// The enums's (unqualified) type name.
    async fn name(&self) -> &str {
        &self.name
    }

    /// Abilities this enum has.
    async fn abilities(&self) -> Option<&Vec<MoveAbility>> {
        Some(&self.abilities)
    }

    /// Constraints on the enum's formal type parameters.  Move bytecode does not name type
    /// parameters, so when they are referenced (e.g. in field types) they are identified by their
    /// index in this list.
    async fn type_parameters(&self) -> Option<&Vec<MoveStructTypeParameter>> {
        Some(&self.type_parameters)
    }

    /// The names and types of the enum's fields.  Field types reference type parameters, by their
    /// index in the defining enum's `typeParameters` list.
    async fn variants(&self) -> Option<&Vec<MoveEnumVariant>> {
        Some(&self.variants)
    }
}

#[Object]
impl MoveEnumVariant {
    /// The name of the variant
    async fn name(&self) -> &str {
        &self.name
    }

    /// The names and types of the variant's fields.  Field types reference type parameters, by their
    /// index in the defining enums's `typeParameters` list.
    async fn fields(&self) -> Option<&Vec<MoveField>> {
        Some(&self.fields)
    }
}

impl MoveEnum {
    pub(crate) fn new(
        module: String,
        name: String,
        def: EnumDef,
        checkpoint_viewed_at: u64,
    ) -> Self {
        let type_parameters = def
            .type_params
            .into_iter()
            .map(|param| MoveStructTypeParameter {
                constraints: abilities(param.constraints),
                is_phantom: param.is_phantom,
            })
            .collect();

        let variants = def
            .variants
            .into_iter()
            .map(|VariantDef { name, signatures }| MoveEnumVariant {
                name,
                fields: signatures
                    .into_iter()
                    .map(|(name, signature)| MoveField {
                        name,
                        type_: signature.into(),
                    })
                    .collect(),
            })
            .collect();

        MoveEnum {
            defining_id: SuiAddress::from(def.defining_id),
            module,
            name,
            abilities: abilities(def.abilities),
            type_parameters,
            variants,
            checkpoint_viewed_at,
        }
    }
}
