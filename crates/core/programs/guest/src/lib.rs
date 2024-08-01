use mugraph_core::prover::Claim;

pub fn apply(_input: u8) -> Claim<u8, u8> {
    // 1. Verify inputs and outputs have same combination of assets
    // 2. Verify inputs and outputs have same amount per asset
    // 3. Verify input contains data for all input proofs
    todo!()
}

pub fn commit(_input: u8) -> Claim<u8, u8> {
    // 1. Verify transaction hash matches mithril certificate
    // 2. Verify transaction hash matches provided pre-image
    // 3. Generate notes for each utxo created
    todo!()
}

pub fn decommit(_input: u8) -> Claim<u8, u8> {
    todo!()
}
