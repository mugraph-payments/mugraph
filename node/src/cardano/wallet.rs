use color_eyre::eyre::Result;
use mugraph_core::types::CardanoWallet;

use super::{
    build_script_address,
    compile_validator,
    compute_script_hash,
    generate_payment_keypair,
    import_payment_key,
};

/// Create or load Cardano wallet
pub async fn setup_cardano_wallet(
    network: &str,
    payment_sk: Option<&str>,
) -> Result<CardanoWallet> {
    let (sk, vk) = if let Some(hex_sk) = payment_sk {
        import_payment_key(hex_sk)?
    } else {
        generate_payment_keypair()?
    };

    let cbor = compile_validator()?;
    let script_hash = compute_script_hash(&cbor);
    let script_address = build_script_address(&script_hash, network)?;

    Ok(CardanoWallet::new(
        sk,
        vk,
        cbor,
        script_hash,
        script_address,
        network.to_string(),
    ))
}
