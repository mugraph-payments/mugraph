use miette::Diagnostic;
use thiserror::Error;

use crate::types::Hash;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Invalid scalar bytes")]
    #[diagnostic(
        code(mugraph_core::invalid_scalar),
        help("Ensure that the scalar bytes are 32 bytes long and represent a valid scalar value for the curve25519 field. This error might occur when creating a Scalar from bytes.")
    )]
    InvalidScalar,

    #[error("Invalid point compression")]
    #[diagnostic(
        code(mugraph_core::invalid_point_compression),
        help("The compressed point format is not valid. Verify that you are using a valid RistrettoPoint compression. This error might occur when compressing or decompressing RistrettoPoints.")
    )]
    InvalidPointCompression,

    #[error("Invalid signature")]
    #[diagnostic(
        code(mugraph_core::invalid_signature),
        help("The signature verification failed. Verify that the signature was created with the correct private key and message. Review the verify function in the schnorr module.")
    )]
    InvalidSignature,

    #[error("Invalid DLEQ proof")]
    #[diagnostic(
        code(mugraph_core::invalid_dleq_proof),
        help("The Discrete Logarithm Equality (DLEQ) proof is not valid. Verify that the proof was generated correctly and with the correct parameters. This error might occur in the verify_dleq_proof function.")
    )]
    InvalidDLEQProof,

    #[error("Invalid unblinded point")]
    #[diagnostic(
        code(mugraph_core::invalid_unblinded_point),
        help("The unblinded point is not valid. Verify that the unblinding process was performed correctly and with the correct parameters. This error might occur in the verify_unblinded_point function.")
    )]
    InvalidUnblindedPoint,

    #[error("Range proof error: {0}")]
    #[diagnostic(
        code(mugraph_core::range_proof_error),
        help("An error occurred during the range proof process. Review the specific error message for more details. This error might occur when creating or verifying range proofs.")
    )]
    RangeProofError(String),

    #[error("Invalid amount")]
    #[diagnostic(
        code(mugraph_core::invalid_amount),
        help(
            "The amount is not valid. Ensure it's a non-negative integer within the allowed range."
        )
    )]
    InvalidAmount,

    #[error("Balance check failed")]
    #[diagnostic(
        code(mugraph_core::balance_check_failed),
        help("The balance check for the transaction failed. Ensure that the sum of inputs equals the sum of outputs for each asset.")
    )]
    BalanceCheckFailed,

    #[error("Bulletproof verification failed")]
    #[diagnostic(
        code(mugraph_core::bulletproof_verification_failed),
        help("The verification of the Bulletproof range proof failed. This could indicate an invalid proof or incorrect verification parameters.")
    )]
    BulletproofVerificationFailed,

    #[error("Invalid commitment")]
    #[diagnostic(
        code(mugraph_core::invalid_commitment),
        help("The commitment is not valid. Ensure all components are correctly formed.")
    )]
    InvalidCommitment,

    #[error("Mismatched input lengths")]
    #[diagnostic(
        code(mugraph_core::mismatched_input_lengths),
        help("The input arrays (asset_ids, amounts, blindings) have mismatched lengths. Ensure all input arrays have the same length.")
    )]
    MismatchedInputLengths,

    #[error("Range proof error: {0}")]
    #[diagnostic(
        code(mugraph_core::range_proof_error),
        help("An error occurred during the range proof process. Review the specific error message for more details.")
    )]
    BulletproofError(#[from] bulletproofs::ProofError),

    #[error("Too many inputs ({0})")]
    #[diagnostic(
        code(mugraph_core::too_many_inputs),
        help("The transaction has exceeded the maximum allowed number of inputs ({0}). Check the transaction construction and ensure it adheres to the protocol limits for inputs.")
    )]
    TooManyInputs(usize),

    #[error("Too many outputs ({0})")]
    #[diagnostic(
        code(mugraph_core::too_many_outputs),
        help("The transaction has exceeded the maximum allowed number of outputs ({0}). Check the transaction construction and ensure it adheres to the protocol limits for outputs.")
    )]
    TooManyOutputs(usize),

    #[error("Transaction does not clear to zero: {0:?}")]
    #[diagnostic(
        code(mugraph_core::transaction_not_clear),
        help("The transaction does not clear to zero for at least one asset. Ensure the transaction is balanced.")
    )]
    TransactionNotClear(Hash),

    #[error("Zero amount is not allowed")]
    #[diagnostic(
        code(mugraph_core::zero_amount),
        help("A zero amount was provided, which is not allowed. Ensure all amounts in the transaction are greater than zero.")
    )]
    ZeroAmount,

    #[error("Invalid proof")]
    #[diagnostic(
        code(mugraph_core::invalid_proof),
        help("The provided zero-knowledge proof is invalid. Ensure that the proof was generated correctly and matches the given swap data.")
    )]
    InvalidProof,

    #[error("Unknown error: {0}")]
    #[diagnostic(
        code(mugraph_core::unknown_error),
        help("An unexpected error occurred. Please check the error details and report this issue if it persists.")
    )]
    UnknownError(#[from] anyhow::Error),

    #[error("Too many assets ({0})")]
    #[diagnostic(
        code(mugraph_core::too_many_assets),
        help("The transaction has exceeded the maximum allowed number of unique assets ({0}). The limit is 8 unique assets per transaction.")
    )]
    TooManyAssets(usize),
}
