use redb::{Key, Value};
use serde::{Deserialize, Serialize};

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

/// Withdrawal record for idempotent signing
/// Key is network[1] + tx_hash[32]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalRecord {
    /// Whether withdrawal has been processed
    pub processed: bool,
    /// Timestamp when withdrawal was recorded
    pub timestamp: u64,
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
    pub fn new(processed: bool) -> Self {
        Self {
            processed,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// UTxO identifier (tx_hash + index)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        bincode::deserialize(data).expect("Failed to deserialize CardanoWallet")
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
        bincode::deserialize(data).expect("Failed to deserialize DepositRecord")
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
        bincode::deserialize(data).expect("Failed to deserialize WithdrawalRecord")
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
        let bytes: &[u8; 34] = data.try_into().expect("Invalid UtxoRef bytes length");
        Self::from_bytes(bytes)
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
        let bytes: &[u8; 33] = data.try_into().expect("Invalid WithdrawalKey bytes length");
        Self::from_bytes(bytes)
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
    use super::*;

    #[test]
    fn test_utxo_ref_serialization() {
        let tx_hash = [1u8; 32];
        let index = 42u16;
        let utxo = UtxoRef::new(tx_hash, index);

        let bytes = utxo.to_bytes();
        let recovered = UtxoRef::from_bytes(&bytes);

        assert_eq!(utxo.tx_hash, recovered.tx_hash);
        assert_eq!(utxo.index, recovered.index);
    }

    #[test]
    fn test_withdrawal_key_serialization() {
        let network = 1u8;
        let tx_hash = [2u8; 32];
        let key = WithdrawalKey::new(network, tx_hash);

        let bytes = key.to_bytes();
        let recovered = WithdrawalKey::from_bytes(&bytes);

        assert_eq!(key.network, recovered.network);
        assert_eq!(key.tx_hash, recovered.tx_hash);
    }
}
