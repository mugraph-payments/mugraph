use bulletproofs;
use miette::Diagnostic;
use std::io;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Failed to hash data")]
    #[diagnostic(
        code(mugraph_core::hash_error),
        help("Verify that the input data is valid and try again. This error might occur in the hash_to_scalar or hash_to_curve functions.")
    )]
    HashError,

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

    #[error("Failed to unblind signature")]
    #[diagnostic(
        code(mugraph_core::unblind_signature_error),
        help("The unblinding process failed. Verify that the blinding factor and other parameters are correct. This error might occur in the unblind_and_verify_signature function.")
    )]
    UnblindSignatureError,

    #[error("Invalid unblinded point")]
    #[diagnostic(
        code(mugraph_core::invalid_unblinded_point),
        help("The unblinded point is not valid. Verify that the unblinding process was performed correctly and with the correct parameters. This error might occur in the verify_unblinded_point function.")
    )]
    InvalidUnblindedPoint,

    #[error("Commitment error: {0}")]
    #[diagnostic(
        code(mugraph_core::commitment_error),
        help("An error occurred during the commitment process. Review the specific error message for more details. This error might occur in the commitment module.")
    )]
    CommitmentError(String),

    #[error("Range proof error: {0}")]
    #[diagnostic(
        code(mugraph_core::range_proof_error),
        help("An error occurred during the range proof process. Review the specific error message for more details. This error might occur when creating or verifying range proofs.")
    )]
    RangeProofError(String),

    #[error("Invalid transaction input")]
    #[diagnostic(
        code(mugraph_core::invalid_transaction_input),
        help("The transaction input is not valid. Verify that all required fields (asset_id, amount, and proof) are present and correctly formatted.")
    )]
    InvalidTransactionInput,

    #[error("Invalid transaction output")]
    #[diagnostic(
        code(mugraph_core::invalid_transaction_output),
        help("The transaction output is not valid. Verify that all required fields (asset_id, amount, and blinded_message) are present and correctly formatted.")
    )]
    InvalidTransactionOutput,

    #[error("Cryptographic operation failed")]
    #[diagnostic(
        code(mugraph_core::crypto_operation_failed),
        help("A cryptographic operation failed. This could be due to invalid input parameters or an internal error in the cryptographic library. Review the input parameters and ensure they are valid.")
    )]
    CryptoOperationFailed,

    #[error("Invalid blinding factor")]
    #[diagnostic(
        code(mugraph_core::invalid_blinding_factor),
        help(
            "The blinding factor is not valid. Ensure that it's a properly generated Scalar value."
        )
    )]
    InvalidBlindingFactor,

    #[error("Invalid asset ID")]
    #[diagnostic(
        code(mugraph_core::invalid_asset_id),
        help("The asset ID is not valid. Ensure it's a 32-byte array representing a valid asset identifier.")
    )]
    InvalidAssetId,

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

    #[error("Invalid transaction commitment")]
    #[diagnostic(
        code(mugraph_core::invalid_transaction_commitment),
        help("The transaction commitment is not valid. Ensure all components (bulletproofs, commitments, asset_ids) are correctly formed.")
    )]
    InvalidTransactionCommitment,

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

    #[error("IO error: {0}")]
    #[diagnostic(
        code(mugraph_core::io_error),
        help(
            "An I/O error occurred. This might be due to file system issues or network problems."
        )
    )]
    IoError(#[from] io::Error),

    #[error("Unexpected error: {0}")]
    #[diagnostic(
        code(mugraph_core::unexpected_error),
        help("An unexpected error occurred. Please report this issue with the error details.")
    )]
    UnexpectedError(String),
}
