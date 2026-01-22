// Cardano key hashes are Blake2b-224 = 28 bytes
pub type DepositDatum {
    user_pubkey_hash: ByteArray,
    node_pubkey_hash: ByteArray,
}

// Ed25519 signature = 64 bytes
pub type SpendRedeemer {
    node_signature: ByteArray,
}
