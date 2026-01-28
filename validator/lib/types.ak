// Cardano key hashes are Blake2b-224 = 28 bytes
pub type DepositDatum {
  user_pubkey_hash: ByteArray,
  node_pubkey_hash: ByteArray,
  // Intent hash stored for reference (verified off-chain only)
  intent_hash: ByteArray,
}

// Simplified redeemer - no longer used for validation
// All validation now happens through transaction witnesses
pub type SpendRedeemer {
  Void
}
