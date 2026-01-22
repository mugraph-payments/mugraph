use cardano/transaction.{Transaction, OutputReference}
use cardano/certificate.{Certificate}
use cardano/governance.{ProposalProcedure, Voter}
use aiken/crypto
use aiken/list
use types.{DepositDatum, SpendRedeemer}

validator deposit {
  spend(datum: Option<DepositDatum>, redeemer: SpendRedeemer, _utxo: OutputReference, self: Transaction) {
    // Must have datum
    expect Some(d) = datum
    
    // Verify node signature against stored node_pubkey_hash
    trace @"Verifying node signature"
    let node_valid = crypto.verify_ed25519_signature(
      d.node_pubkey_hash,
      self.id,
      redeemer.node_signature,
    )
    
    // Verify user signature is in extra_signatories
    trace @"Verifying user signature"
    let user_signed = list.any(self.extra_signatories, fn(sig) { sig == d.user_pubkey_hash })
    
    // Both must be true
    trace @"Checking both signatures valid"
    node_valid && user_signed
  }

  withdraw(_redeemer: Data, _account: Data, _self: Transaction) {
    trace @"Withdraw purpose not supported"
    False
  }
  
  publish(_redeemer: Data, _certificate: Certificate, _self: Transaction) {
    trace @"Publish purpose not supported"
    False
  }
  
  vote(_redeemer: Data, _voter: Voter, _self: Transaction) {
    trace @"Vote purpose not supported"
    False
  }
  
  propose(_redeemer: Data, _proposal: ProposalProcedure, _self: Transaction) {
    trace @"Propose purpose not supported"
    False
  }
  
  mint(_redeemer: Data, _policy_id: Data, _self: Transaction) {
    trace @"Mint purpose not supported"
    False
  }
}
