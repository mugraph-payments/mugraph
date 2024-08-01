use mugraph_core::prover::Claim;

pub fn commit(_input: u8) -> Claim<u8, u8> {
    // 1. Verify transaction hash matches mithril certificate
    // 2. Verify transaction hash matches provided pre-image
    // 3. Generate notes for each utxo created
    todo!()
}
