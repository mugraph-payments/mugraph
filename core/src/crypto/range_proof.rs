use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand_core::{CryptoRng, RngCore};

use crate::crypto::{hash_to_curve, G, H};
use crate::Hash;

pub struct AssetCommitment {
    pub commitment: CompressedRistretto,
    pub asset_proof: AssetProof,
    pub range_proof: RangeProof,
}

pub struct AssetProof {
    pub t: CompressedRistretto,
    pub s: Scalar,
    pub asset_id_commitment: CompressedRistretto,
}

pub fn create_asset_commitment<R: RngCore + CryptoRng>(
    rng: &mut R,
    asset_id: &Hash,
    amount: u64,
    blinding: Scalar,
) -> Result<AssetCommitment, &'static str> {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, 1);

    // Create the commitment
    let h_a = hash_to_curve(asset_id);
    let commitment = (*G * Scalar::from(amount) + *H * blinding + h_a).compress();

    // Create the range proof
    let mut prover_transcript = Transcript::new(b"AssetRangeProof");
    let (range_proof, _) = RangeProof::prove_single(
        &bp_gens,
        &pc_gens,
        &mut prover_transcript,
        amount,
        &blinding,
        64,
    )
    .map_err(|_| "Failed to create range proof")?;

    // Create the asset proof
    let r = Scalar::random(rng);
    let asset_id_commitment = (*G * Scalar::from_bytes_mod_order(*asset_id) + *H * r).compress();
    let t = (h_a * blinding).compress();
    let challenge = compute_challenge(&commitment, &asset_id_commitment, &t);
    let s = r + challenge * blinding;

    Ok(AssetCommitment {
        commitment,
        asset_proof: AssetProof {
            t,
            s,
            asset_id_commitment,
        },
        range_proof,
    })
}

pub fn verify_asset_commitment(
    commitment: &AssetCommitment,
    asset_id: &Hash,
) -> Result<(), &'static str> {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, 1);

    // Verify the range proof
    let mut verifier_transcript = Transcript::new(b"AssetRangeProof");
    commitment
        .range_proof
        .verify_single(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &commitment.commitment,
            64,
        )
        .map_err(|_| "Range proof verification failed")?;

    // Verify the asset proof
    let h_a = hash_to_curve(asset_id);
    let challenge = compute_challenge(
        &commitment.commitment,
        &commitment.asset_proof.asset_id_commitment,
        &commitment.asset_proof.t,
    );
    let lhs = *G * commitment.asset_proof.s + h_a * challenge;
    let rhs = commitment
        .asset_proof
        .asset_id_commitment
        .decompress()
        .unwrap()
        + commitment.asset_proof.t.decompress().unwrap() * challenge;

    if lhs != rhs {
        return Err("Asset proof verification failed");
    }

    Ok(())
}

fn compute_challenge(
    commitment: &CompressedRistretto,
    asset_id_commitment: &CompressedRistretto,
    t: &CompressedRistretto,
) -> Scalar {
    let mut transcript = Transcript::new(b"AssetProofChallenge");
    transcript.append_message(b"commitment", commitment.as_bytes());
    transcript.append_message(b"asset_id_commitment", asset_id_commitment.as_bytes());
    transcript.append_message(b"t", t.as_bytes());
    let mut challenge_bytes = [0u8; 64];
    transcript.challenge_bytes(b"challenge", &mut challenge_bytes);
    Scalar::from_bytes_mod_order_wide(&challenge_bytes)
}
