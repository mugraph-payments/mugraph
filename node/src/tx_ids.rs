use mugraph_core::{
    error::Error,
    types::{UtxoRef, WithdrawalKey},
};

pub fn parse_tx_hash(tx_hash: &str) -> Result<[u8; 32], Error> {
    let tx_hash = hex::decode(tx_hash).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid tx_hash hex: {}", e),
    })?;

    tx_hash.try_into().map_err(|_| Error::InvalidInput {
        reason: "tx_hash must be 32 bytes".to_string(),
    })
}

pub fn parse_utxo_ref(tx_hash: &str, index: u16) -> Result<UtxoRef, Error> {
    Ok(UtxoRef::new(parse_tx_hash(tx_hash)?, index))
}

pub fn parse_withdrawal_key(
    tx_hash: &str,
    network_byte: u8,
) -> Result<WithdrawalKey, Error> {
    Ok(WithdrawalKey::new(network_byte, parse_tx_hash(tx_hash)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tx_hash_rejects_malformed_hex_and_wrong_length() {
        let malformed = parse_tx_hash("not-hex").unwrap_err();
        assert!(matches!(malformed, Error::InvalidInput { .. }));
        assert!(format!("{malformed:?}").contains("Invalid tx_hash hex"));

        let wrong_length = parse_tx_hash(&"ab".repeat(31)).unwrap_err();
        assert!(matches!(wrong_length, Error::InvalidInput { .. }));
        assert!(
            format!("{wrong_length:?}").contains("tx_hash must be 32 bytes")
        );

        let parsed = parse_tx_hash(&"cd".repeat(32))
            .expect("32-byte tx hash must parse");
        assert_eq!(parsed, [0xcdu8; 32]);
    }

    #[test]
    fn parsed_utxo_ref_and_withdrawal_key_match_existing_construction() {
        let tx_hash_hex = "ab".repeat(32);
        let expected_tx_hash = [0xabu8; 32];

        let utxo_ref = parse_utxo_ref(&tx_hash_hex, 7).expect("utxo ref");
        assert_eq!(utxo_ref, UtxoRef::new(expected_tx_hash, 7));

        let withdrawal_key =
            parse_withdrawal_key(&tx_hash_hex, 3).expect("withdrawal key");
        assert_eq!(withdrawal_key, WithdrawalKey::new(3, expected_tx_hash));
    }
}
