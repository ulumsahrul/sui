// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::base_types::ObjectID;
use crate::base_types::SequenceNumber;
use crate::error::SuiResult;
use crate::object::Owner;
use crate::storage::ObjectStore;
use crate::sui_serde::BigInt;
use crate::sui_serde::Readable;
use crate::versioned::Versioned;
use crate::SUI_BRIDGE_OBJECT_ID;
use enum_dispatch::enum_dispatch;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::collection_types::LinkedTableNode;
use crate::dynamic_field::{get_dynamic_field_from_store, Field};
use crate::{
    base_types::SuiAddress,
    collection_types::{Bag, LinkedTable, VecMap},
    error::SuiError,
    id::UID,
};

pub type BridgeInnerDynamicField = Field<u64, BridgeInnerV1>;
pub type BridgeRecordDyanmicField = Field<
    MoveTypeBridgeMessageKey,
    LinkedTableNode<MoveTypeBridgeMessageKey, MoveTypeBridgeRecord>,
>;

pub const BRIDGE_MODULE_NAME: &IdentStr = ident_str!("bridge");
pub const BRIDGE_CREATE_FUNCTION_NAME: &IdentStr = ident_str!("create");

pub const BRIDGE_SUPPORTED_ASSET: &[&str] = &["btc", "eth", "usdc", "usdt"];

pub fn get_bridge_obj_initial_shared_version(
    object_store: &dyn ObjectStore,
) -> SuiResult<Option<SequenceNumber>> {
    Ok(object_store
        .get_object(&SUI_BRIDGE_OBJECT_ID)?
        .map(|obj| match obj.owner {
            Owner::Shared {
                initial_shared_version,
            } => initial_shared_version,
            _ => unreachable!("Bridge object must be shared"),
        }))
}

/// Bridge provides an abstraction over multiple versions of the inner BridgeInner object.
/// This should be the primary interface to the bridge object in Rust.
/// We use enum dispatch to dispatch all methods defined in BridgeTrait to the actual
/// implementation in the inner types.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_dispatch(BridgeTrait)]
pub enum Bridge {
    V1(BridgeInnerV1),
}

/// Rust version of the Move sui::bridge::Bridge type
/// This repreents the object with 0x9 ID.
/// In Rust, this type should be rarely used since it's just a thin
/// wrapper used to access the inner object.
/// Within this module, we use it to determine the current version of the bridge inner object type,
/// so that we could deserialize the inner object correctly.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BridgeWrapper {
    pub id: UID,
    pub version: Versioned,
}

/// This is the standard API that all bridge inner object type should implement.
#[enum_dispatch]
pub trait BridgeTrait {
    fn message_version(&self) -> u64;
    fn chain_id(&self) -> u8;
    fn sequence_nums(&self) -> &VecMap<u8, u64>;
    fn committee(&self) -> &MoveTypeBridgeCommittee;
    fn treasury(&self) -> &MoveTypeBridgeTreasury;
    fn bridge_records(&self) -> &LinkedTable<MoveTypeBridgeMessageKey>;
    fn frozen(&self) -> bool;
    fn into_bridge_summary(self) -> BridgeSummary;
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeSummary {
    // Message version
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "Readable<BigInt<u64>, _>")]
    pub message_version: u64,
    /// Self Chain ID
    pub chain_id: u8,
    /// Sequence numbers of all message types
    #[schemars(with = "Vec<(u8, BigInt<u64>)>")]
    #[serde_as(as = "Vec<(_, Readable<BigInt<u64>, _>)>")]
    pub sequence_nums: Vec<(u8, u64)>,
    pub committee: BridgeCommitteeSummary,
    /// Object ID of bridge Records (dynamic field)
    pub bridge_records_id: ObjectID,
    /// Whether the bridge is currently frozen or not
    pub is_frozen: bool,
    // TODO: add treasury
    // TODO: add limiter
}

pub fn get_bridge_wrapper(object_store: &dyn ObjectStore) -> Result<BridgeWrapper, SuiError> {
    let wrapper = object_store
        .get_object(&SUI_BRIDGE_OBJECT_ID)?
        // Don't panic here on None because object_store is a generic store.
        .ok_or_else(|| SuiError::SuiBridgeReadError("BridgeWrapper object not found".to_owned()))?;
    let move_object = wrapper.data.try_as_move().ok_or_else(|| {
        SuiError::SuiBridgeReadError("BridgeWrapper object must be a Move object".to_owned())
    })?;
    let result = bcs::from_bytes::<BridgeWrapper>(move_object.contents())
        .map_err(|err| SuiError::SuiBridgeReadError(err.to_string()))?;
    Ok(result)
}

pub fn get_bridge(object_store: &dyn ObjectStore) -> Result<Bridge, SuiError> {
    let wrapper = get_bridge_wrapper(object_store)?;
    let id = wrapper.version.id.id.bytes;
    let version = wrapper.version.version;
    match version {
        1 => {
            let result: BridgeInnerV1 = get_dynamic_field_from_store(object_store, id, &version)
                .map_err(|err| {
                    SuiError::DynamicFieldReadError(format!(
                        "Failed to load bridge inner object with ID {:?} and version {:?}: {:?}",
                        id, version, err
                    ))
                })?;
            Ok(Bridge::V1(result))
        }
        _ => Err(SuiError::SuiBridgeReadError(format!(
            "Unsupported SuiBridge version: {}",
            version
        ))),
    }
}

/// Rust version of the Move bridge::BridgeInner type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BridgeInnerV1 {
    pub message_version: u64,
    pub chain_id: u8,
    pub sequence_nums: VecMap<u8, u64>,
    pub committee: MoveTypeBridgeCommittee,
    pub treasury: MoveTypeBridgeTreasury,
    pub bridge_records: LinkedTable<MoveTypeBridgeMessageKey>,
    pub limiter: MoveTypeBridgeTransferLimiter,
    pub frozen: bool,
}

impl BridgeTrait for BridgeInnerV1 {
    fn message_version(&self) -> u64 {
        self.message_version
    }

    fn chain_id(&self) -> u8 {
        self.chain_id
    }

    fn sequence_nums(&self) -> &VecMap<u8, u64> {
        &self.sequence_nums
    }

    fn committee(&self) -> &MoveTypeBridgeCommittee {
        &self.committee
    }

    fn treasury(&self) -> &MoveTypeBridgeTreasury {
        &self.treasury
    }

    fn bridge_records(&self) -> &LinkedTable<MoveTypeBridgeMessageKey> {
        &self.bridge_records
    }

    fn frozen(&self) -> bool {
        self.frozen
    }

    fn into_bridge_summary(self) -> BridgeSummary {
        BridgeSummary {
            message_version: self.message_version,
            chain_id: self.chain_id,
            sequence_nums: self
                .sequence_nums
                .contents
                .into_iter()
                .map(|e| (e.key, e.value))
                .collect(),
            committee: BridgeCommitteeSummary {
                members: self
                    .committee
                    .members
                    .contents
                    .into_iter()
                    .map(|e| (e.key, e.value))
                    .collect(),
                thresholds: self
                    .committee
                    .thresholds
                    .contents
                    .into_iter()
                    .map(|e| (e.key, e.value))
                    .collect(),
            },
            bridge_records_id: self.bridge_records.id,
            is_frozen: self.frozen,
        }
    }
}

/// Rust version of the Move treasury::BridgeTreasury type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeTreasury {
    pub treasuries: Bag,
}

/// Rust version of the Move committee::BridgeCommittee type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeCommittee {
    pub members: VecMap<Vec<u8>, MoveTypeCommitteeMember>,
    pub thresholds: VecMap<u8, u64>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BridgeCommitteeSummary {
    pub members: Vec<(Vec<u8>, MoveTypeCommitteeMember)>,
    pub thresholds: Vec<(u8, u64)>,
}

/// Rust version of the Move committee::CommitteeMember type.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveTypeCommitteeMember {
    pub sui_address: SuiAddress,
    pub bridge_pubkey_bytes: Vec<u8>,
    pub voting_power: u64,
    pub http_rest_url: Vec<u8>,
    pub blocklisted: bool,
}

/// Rust version of the Move message::BridgeMessageKey type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeMessageKey {
    pub source_chain: u8,
    pub message_type: u8,
    pub bridge_seq_num: u64,
}

/// Rust version of the Move limiter::TransferLimiter type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeTransferLimiter {
    pub transfer_limit: VecMap<MoveTypeBridgeRoute, u64>,
    pub notional_values: VecMap<u8, u64>,
    pub transfer_records: VecMap<MoveTypeBridgeRoute, MoveTypeBridgeTransferRecord>,
}

/// Rust version of the Move chain_ids::BridgeRoute type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeRoute {
    pub source: u8,
    pub destination: u8,
}

/// Rust version of the Move limiter::TransferRecord type.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTypeBridgeTransferRecord {
    hour_head: u64,
    hour_tail: u64,
    per_hour_amounts: Vec<u64>,
    total_amount: u64,
}

/// Rust version of the Move message::BridgeMessage type.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveTypeBridgeMessage {
    pub message_type: u8,
    pub message_version: u8,
    pub seq_num: u64,
    pub source_chain: u8,
    pub payload: Vec<u8>,
}

/// Rust version of the Move message::BridgeMessage type.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveTypeBridgeRecord {
    pub message: MoveTypeBridgeMessage,
    pub verified_signatures: Option<Vec<Vec<u8>>>,
    pub claimed: bool,
}
