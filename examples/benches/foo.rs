use calliper::{utils::black_box, Runner, Scenario};
use mugraph_core::types::Hash;
use smallvec::SmallVec;

#[derive(Default)]
pub struct Transaction {
    pub inputs: Inputs<4>,
    pub outputs: Outputs<4>,
    pub data: SmallVec<[[u8; 256]; 8]>,
    pub assets: SmallVec<[Hash; 4]>,
}

#[derive(Default)]
pub struct Inputs<const N: usize> {
    pub parents: SmallVec<[Hash; N]>,
    pub indexes: SmallVec<[u8; N]>,
    pub asset_ids: SmallVec<[u32; N]>,
    pub amounts: SmallVec<[u64; N]>,
    pub program_id: SmallVec<[Option<Hash>; N]>,
    pub data: SmallVec<[Option<u32>; N]>,
}

#[derive(Default)]
pub struct Outputs<const N: usize> {
    pub asset_ids: SmallVec<[u32; N]>,
    pub amounts: SmallVec<[u64; N]>,
    pub program_id: SmallVec<[Option<Hash>; N]>,
    pub data: SmallVec<[Option<u32>; N]>,
}

pub struct Seal {
    pub nullifiers: SmallVec<[Hash; 4]>,
}

#[inline(never)]
#[no_mangle]
fn validate(transaction: Transaction) {
    let mut input_amounts = [0u64; 4];
    let mut output_amounts = [0u64; 4];

    for (asset_id, amount) in transaction
        .inputs
        .asset_ids
        .iter()
        .zip(transaction.inputs.amounts.iter())
    {
        let index = transaction
            .assets
            .iter()
            .position(|&a| a.as_bytes()[0] == *asset_id as u8)
            .unwrap_or(0);
        input_amounts[index] = input_amounts[index].wrapping_add(*amount);
    }

    for (asset_id, amount) in transaction
        .outputs
        .asset_ids
        .iter()
        .zip(transaction.outputs.amounts.iter())
    {
        let index = transaction
            .assets
            .iter()
            .position(|&a| a.as_bytes()[0] == *asset_id as u8)
            .unwrap_or(0);
        output_amounts[index] = output_amounts[index].wrapping_add(*amount);
    }

    let mut is_valid = true;

    for i in 0..4 {
        is_valid &= input_amounts[i] == output_amounts[i];
    }

    assert!(
        is_valid,
        "Input and output amounts do not match for all asset IDs"
    );
}

fn bench_validate() {
    let mut transaction = Transaction::default();

    transaction
        .inputs
        .parents
        .extend_from_slice(&[Hash::default(); 4]);
    transaction.inputs.indexes.extend_from_slice(&[0, 1, 2, 3]);
    transaction
        .inputs
        .asset_ids
        .extend_from_slice(&[1, 2, 3, 1]);
    transaction
        .inputs
        .amounts
        .extend_from_slice(&[100, 200, 300, 400]);
    transaction.inputs.program_id.extend_from_slice(&[None; 4]);
    transaction.inputs.data.extend_from_slice(&[None; 4]);

    transaction
        .outputs
        .asset_ids
        .extend_from_slice(&[1, 2, 3, 1]);
    transaction
        .outputs
        .amounts
        .extend_from_slice(&[150, 200, 300, 350]);
    transaction.outputs.program_id.extend_from_slice(&[None; 4]);
    transaction.outputs.data.extend_from_slice(&[None; 4]);

    transaction
        .assets
        .extend_from_slice(&[[1; 32].into(), [2; 32].into(), [3; 32].into()]);

    black_box(validate(transaction));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = Runner::default();
    let benches = [Scenario::new(bench_validate)];

    if let Some(results) = runner.run(&benches)? {
        for res in results.into_iter() {
            println!("{}", res.parse());
        }
    }

    Ok(())
}
