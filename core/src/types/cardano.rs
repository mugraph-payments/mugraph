use redb::{Key, Value};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::types::{TransferChainState, TransferCreditState};

/// Cardano wallet data stored in the database
/// Contains node keys and validator script artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardanoWallet {
    /// Node payment signing key (32 bytes)
    pub payment_sk: Vec<u8>,
    /// Node payment verification key (32 bytes)
    pub payment_vk: Vec<u8>,
    /// Aiken validator script CBOR
    pub script_cbor: Vec<u8>,
    /// Script hash (28 bytes for Blake2b-224)
    pub script_hash: Vec<u8>,
    /// Script address (bech32 encoded)
    pub script_address: String,
    /// Network identifier (mainnet/testnet/preprod/preview)
    pub network: String,
    /// Timestamp when validator was compiled
    pub compiled_at: u64,
}

/// Deposit record tracking on-chain deposits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRecord {
    /// Whether the deposit has been spent/claimed
    pub spent: bool,
    /// Block height when deposit was recorded
    pub block_height: u64,
    /// Unix timestamp when deposit was created
    pub created_at: u64,
    /// Unix timestamp when deposit expires
    pub expires_at: u64,
    /// Intent hash (blake2b-256 of canonical deposit payload) for replay protection
    /// Stored as 32 bytes, empty if not used
    pub intent_hash: [u8; 32],
}

/// Withdrawal status for tracking state machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WithdrawalStatus {
    /// Withdrawal is pending (notes burned, not yet submitted)
    Pending,
    /// Withdrawal completed successfully
    Completed,
    /// Withdrawal failed (submission failed after notes burned)
    Failed,
}

/// Withdrawal record for idempotent signing
/// Key is network[1] + tx_hash[32]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalRecord {
    /// Withdrawal status
    pub status: WithdrawalStatus,
    /// Timestamp when withdrawal was recorded
    pub timestamp: u64,
}

/// Cross-node transfer persistence record (M3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossNodeTransferRecord {
    pub transfer_id: String,
    pub source_node_id: String,
    pub destination_node_id: String,
    pub tx_hash: Option<String>,
    pub chain_state: String,
    pub credit_state: String,
    pub confirmations_observed: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl CrossNodeTransferRecord {
    pub fn parsed_chain_state(&self) -> TransferChainState {
        Self::decode_chain_state(&self.chain_state)
    }

    pub fn parsed_credit_state(&self) -> TransferCreditState {
        Self::decode_credit_state(&self.credit_state)
    }

    pub fn set_chain_state(&mut self, state: TransferChainState) {
        self.chain_state = Self::encode_chain_state(state).to_string();
    }

    pub fn set_credit_state(&mut self, state: TransferCreditState) {
        self.credit_state = Self::encode_credit_state(state).to_string();
    }

    pub fn decode_chain_state(value: &str) -> TransferChainState {
        match value {
            "submitted" => TransferChainState::Submitted,
            "confirming" => TransferChainState::Confirming,
            "confirmed" => TransferChainState::Confirmed,
            "invalidated" => TransferChainState::Invalidated,
            _ => TransferChainState::Unknown,
        }
    }

    pub fn decode_credit_state(value: &str) -> TransferCreditState {
        match value {
            "eligible" => TransferCreditState::Eligible,
            "credited" => TransferCreditState::Credited,
            "held" => TransferCreditState::Held,
            "reversed" => TransferCreditState::Reversed,
            _ => TransferCreditState::None,
        }
    }

    pub fn encode_chain_state(state: TransferChainState) -> &'static str {
        match state {
            TransferChainState::Unknown => "unknown",
            TransferChainState::Submitted => "submitted",
            TransferChainState::Confirming => "confirming",
            TransferChainState::Confirmed => "confirmed",
            TransferChainState::Invalidated => "invalidated",
        }
    }

    pub fn encode_credit_state(state: TransferCreditState) -> &'static str {
        match state {
            TransferCreditState::None => "none",
            TransferCreditState::Eligible => "eligible",
            TransferCreditState::Credited => "credited",
            TransferCreditState::Held => "held",
            TransferCreditState::Reversed => "reversed",
        }
    }
}

/// Cross-node message persistence record (M3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossNodeMessageRecord {
    pub message_id: String,
    pub transfer_id: String,
    pub message_type: String,
    pub direction: String,
    pub attempt_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Idempotency persistence record (M3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdempotencyRecord {
    pub idempotency_key: String,
    pub transfer_id: String,
    pub message_type: String,
    pub request_hash: String,
    pub first_seen_at: u64,
    pub expires_at: u64,
}

/// Audit event persistence record (M3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferAuditEvent {
    pub event_id: String,
    pub transfer_id: String,
    pub event_type: String,
    pub reason: String,
    pub created_at: u64,
}

trait CorruptFallback {
    fn corrupt_fallback() -> Self;
}

fn deserialize_or_fallback<T: DeserializeOwned + CorruptFallback>(
    data: &[u8],
) -> T {
    bincode::deserialize(data).unwrap_or_else(|_| T::corrupt_fallback())
}

impl CorruptFallback for CardanoWallet {
    fn corrupt_fallback() -> Self {
        Self {
            payment_sk: Vec::new(),
            payment_vk: Vec::new(),
            script_cbor: Vec::new(),
            script_hash: Vec::new(),
            script_address: String::new(),
            network: "corrupt".to_string(),
            compiled_at: 0,
        }
    }
}

impl CorruptFallback for DepositRecord {
    fn corrupt_fallback() -> Self {
        Self {
            spent: true,
            block_height: 0,
            created_at: 0,
            expires_at: 0,
            intent_hash: [0u8; 32],
        }
    }
}

impl CorruptFallback for WithdrawalRecord {
    fn corrupt_fallback() -> Self {
        Self {
            status: WithdrawalStatus::Failed,
            timestamp: 0,
        }
    }
}

impl CorruptFallback for CrossNodeTransferRecord {
    fn corrupt_fallback() -> Self {
        Self {
            transfer_id: String::new(),
            source_node_id: String::new(),
            destination_node_id: String::new(),
            tx_hash: None,
            chain_state: "invalidated".to_string(),
            credit_state: "held".to_string(),
            confirmations_observed: 0,
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl CorruptFallback for CrossNodeMessageRecord {
    fn corrupt_fallback() -> Self {
        Self {
            message_id: String::new(),
            transfer_id: String::new(),
            message_type: String::new(),
            direction: "terminal".to_string(),
            attempt_count: u32::MAX,
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl CorruptFallback for IdempotencyRecord {
    fn corrupt_fallback() -> Self {
        Self {
            idempotency_key: String::new(),
            transfer_id: String::new(),
            message_type: String::new(),
            request_hash: String::new(),
            first_seen_at: 0,
            expires_at: 0,
        }
    }
}

impl CorruptFallback for TransferAuditEvent {
    fn corrupt_fallback() -> Self {
        Self {
            event_id: String::new(),
            transfer_id: String::new(),
            event_type: "corrupt_record".to_string(),
            reason: "failed to deserialize persisted audit event".to_string(),
            created_at: 0,
        }
    }
}

impl CardanoWallet {
    pub fn new(
        payment_sk: Vec<u8>,
        payment_vk: Vec<u8>,
        script_cbor: Vec<u8>,
        script_hash: Vec<u8>,
        script_address: String,
        network: String,
    ) -> Self {
        Self {
            payment_sk,
            payment_vk,
            script_cbor,
            script_hash,
            script_address,
            network,
            compiled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl DepositRecord {
    pub fn new(block_height: u64, created_at: u64, expires_at: u64) -> Self {
        Self {
            spent: false,
            block_height,
            created_at,
            expires_at,
            intent_hash: [0u8; 32],
        }
    }

    pub fn with_intent_hash(
        block_height: u64,
        created_at: u64,
        expires_at: u64,
        intent_hash: [u8; 32],
    ) -> Self {
        Self {
            spent: false,
            block_height,
            created_at,
            expires_at,
            intent_hash,
        }
    }
}

impl WithdrawalRecord {
    pub fn new(status: WithdrawalStatus) -> Self {
        Self {
            status,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn pending() -> Self {
        Self::new(WithdrawalStatus::Pending)
    }

    pub fn completed() -> Self {
        Self::new(WithdrawalStatus::Completed)
    }

    pub fn failed() -> Self {
        Self::new(WithdrawalStatus::Failed)
    }
}

/// UTxO identifier (tx_hash + index)
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct UtxoRef {
    /// Transaction hash (32 bytes)
    pub tx_hash: [u8; 32],
    /// Output index (2 bytes)
    pub index: u16,
}

impl UtxoRef {
    pub fn new(tx_hash: [u8; 32], index: u16) -> Self {
        Self { tx_hash, index }
    }

    /// Serialize to bytes for database key
    pub fn to_bytes(&self) -> [u8; 34] {
        let mut bytes = [0u8; 34];
        bytes[..32].copy_from_slice(&self.tx_hash);
        bytes[32..34].copy_from_slice(&self.index.to_be_bytes());
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8; 34]) -> Self {
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(&bytes[..32]);
        let index = u16::from_be_bytes([bytes[32], bytes[33]]);
        Self { tx_hash, index }
    }
}

/// Withdrawal key with network discriminator
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct WithdrawalKey {
    /// Network identifier byte
    pub network: u8,
    /// Transaction hash (32 bytes)
    pub tx_hash: [u8; 32],
}

impl WithdrawalKey {
    pub fn new(network: u8, tx_hash: [u8; 32]) -> Self {
        Self { network, tx_hash }
    }

    /// Serialize to bytes for database key
    pub fn to_bytes(&self) -> [u8; 33] {
        let mut bytes = [0u8; 33];
        bytes[0] = self.network;
        bytes[1..].copy_from_slice(&self.tx_hash);
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8; 33]) -> Self {
        let network = bytes[0];
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(&bytes[1..]);
        Self { network, tx_hash }
    }
}

impl Value for CardanoWallet {
    type SelfType<'a> = CardanoWallet;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).expect("Failed to serialize CardanoWallet")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::CardanoWallet")
    }
}

impl Value for DepositRecord {
    type SelfType<'a> = DepositRecord;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).expect("Failed to serialize DepositRecord")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::DepositRecord")
    }
}

impl Value for WithdrawalRecord {
    type SelfType<'a> = WithdrawalRecord;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).expect("Failed to serialize WithdrawalRecord")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::WithdrawalRecord")
    }
}

impl Value for CrossNodeTransferRecord {
    type SelfType<'a> = CrossNodeTransferRecord;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value)
            .expect("Failed to serialize CrossNodeTransferRecord")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::CrossNodeTransferRecord")
    }
}

impl Value for CrossNodeMessageRecord {
    type SelfType<'a> = CrossNodeMessageRecord;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value)
            .expect("Failed to serialize CrossNodeMessageRecord")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::CrossNodeMessageRecord")
    }
}

impl Value for IdempotencyRecord {
    type SelfType<'a> = IdempotencyRecord;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value)
            .expect("Failed to serialize IdempotencyRecord")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::IdempotencyRecord")
    }
}

impl Value for TransferAuditEvent {
    type SelfType<'a> = TransferAuditEvent;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        deserialize_or_fallback(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value)
            .expect("Failed to serialize TransferAuditEvent")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::TransferAuditEvent")
    }
}

impl Key for UtxoRef {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Value for UtxoRef {
    type SelfType<'a> = UtxoRef;
    type AsBytes<'a> = [u8; 34];

    fn fixed_width() -> Option<usize> {
        Some(34)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        match data.try_into() {
            Ok(bytes) => Self::from_bytes(bytes),
            Err(_) => Self {
                tx_hash: [0u8; 32],
                index: 0,
            },
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.to_bytes()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::UtxoRef")
    }
}

impl Key for WithdrawalKey {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Value for WithdrawalKey {
    type SelfType<'a> = WithdrawalKey;
    type AsBytes<'a> = [u8; 33];

    fn fixed_width() -> Option<usize> {
        Some(33)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        match data.try_into() {
            Ok(bytes) => Self::from_bytes(bytes),
            Err(_) => Self {
                network: 0,
                tx_hash: [0u8; 32],
            },
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.to_bytes()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("mugraph::WithdrawalKey")
    }
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn prop_utxo_ref_roundtrip(tx_hash: [u8; 32], index: u16) {
        let original = UtxoRef::new(tx_hash, index);
        let recovered = UtxoRef::from_bytes(&original.to_bytes());
        prop_assert_eq!(original, recovered);
    }

    #[proptest]
    fn prop_withdrawal_key_roundtrip(network: u8, tx_hash: [u8; 32]) {
        let original = WithdrawalKey::new(network, tx_hash);
        let recovered = WithdrawalKey::from_bytes(&original.to_bytes());
        prop_assert_eq!(original, recovered);
    }

    #[test]
    fn malformed_deposit_record_bytes_fail_closed_without_panicking() {
        let value = <DepositRecord as Value>::from_bytes(&[0xff, 0x00, 0x01]);
        assert!(value.spent, "corrupt deposit record must fail closed");
    }

    #[test]
    fn malformed_withdrawal_record_bytes_fail_closed_without_panicking() {
        let value =
            <WithdrawalRecord as Value>::from_bytes(&[0xaa, 0xbb, 0xcc]);
        assert_eq!(value.status, WithdrawalStatus::Failed);
    }

    #[test]
    fn cross_node_record_state_accessors_preserve_legacy_string_encoding() {
        let mut record = CrossNodeTransferRecord {
            transfer_id: "tr-1".to_string(),
            source_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            tx_hash: Some("abcd".to_string()),
            chain_state: "submitted".to_string(),
            credit_state: "held".to_string(),
            confirmations_observed: 3,
            created_at: 1,
            updated_at: 2,
        };

        assert_eq!(record.parsed_chain_state(), TransferChainState::Submitted);
        assert_eq!(record.parsed_credit_state(), TransferCreditState::Held);

        record.set_chain_state(TransferChainState::Invalidated);
        record.set_credit_state(TransferCreditState::Reversed);
        assert_eq!(record.chain_state, "invalidated");
        assert_eq!(record.credit_state, "reversed");

        record.chain_state = "unknown-value".to_string();
        record.credit_state = "unknown-value".to_string();
        assert_eq!(record.parsed_chain_state(), TransferChainState::Unknown);
        assert_eq!(record.parsed_credit_state(), TransferCreditState::None);
    }

    #[test]
    fn malformed_utxo_ref_bytes_do_not_panic() {
        let value = <UtxoRef as Value>::from_bytes(&[1u8; 3]);
        assert_eq!(value.index, 0);
    }
}
