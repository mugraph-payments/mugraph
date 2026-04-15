mod address;
mod keys;
mod validator_artifacts;
mod wallet;

pub use address::{build_script_address, compute_script_hash};
pub use keys::{generate_payment_keypair, import_payment_key};
pub use validator_artifacts::{
    compile_validator,
    get_validator_dir,
    load_validator_cbor,
    validator_artifacts_exist,
};
pub use wallet::setup_cardano_wallet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        assert_eq!(sk.len(), 32);
        assert_eq!(vk.len(), 32);
    }

    #[test]
    fn test_import_key() {
        // Generate a key first
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = hex::encode(&sk);

        // Import it back
        let (imported_sk, imported_vk) = import_payment_key(&hex_sk).unwrap();
        assert_eq!(sk, imported_sk);
        assert_eq!(vk, imported_vk);
    }

    #[test]
    fn import_payment_key_accepts_0x_prefix() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = format!("0x{}", hex::encode(&sk));

        let (imported_sk, imported_vk) = import_payment_key(&hex_sk).unwrap();
        assert_eq!(sk, imported_sk);
        assert_eq!(vk, imported_vk);
    }

    #[test]
    fn import_payment_key_rejects_wrong_length() {
        let err = import_payment_key(&hex::encode([7u8; 31])).unwrap_err();
        assert!(err.to_string().contains("Signing key must be 32 bytes"));
    }

    #[test]
    fn test_script_hash() {
        let cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_script_hash(&cbor);
        assert_eq!(hash.len(), 28); // Blake2b-224
    }

    #[test]
    fn compute_script_hash_matches_known_vector() {
        let cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_script_hash(&cbor);
        assert_eq!(
            hex::encode(hash),
            "7c4412a4936b244f2f1c645bf039c49d57b8cd18108b1a9ae5220a42"
        );
    }

    #[test]
    fn build_script_address_rejects_unknown_network() {
        let err = build_script_address(&[0u8; 28], "staging").unwrap_err();
        assert!(err.to_string().contains("Unknown network: staging"));
    }

    #[tokio::test]
    async fn setup_cardano_wallet_preserves_imported_key_and_network() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = hex::encode(&sk);

        let wallet = setup_cardano_wallet("preprod", Some(&hex_sk))
            .await
            .expect("wallet from imported key");

        assert_eq!(wallet.payment_sk, sk);
        assert_eq!(wallet.payment_vk, vk);
        assert_eq!(wallet.network, "preprod");
        assert_eq!(wallet.script_hash.len(), 28);
        assert!(wallet.script_address.starts_with("addr_test1"));
    }
}
