mod asset;
mod cardano;
mod dleq;
mod hash;
mod hex_array;
mod keypair;
mod note;
mod output;
mod public_key;
mod refresh;
mod request;
mod response;
mod secret_key;
mod signature;
mod xnode;

pub use self::{
    asset::*, cardano::*, dleq::*, hash::*, keypair::*, note::*, output::*,
    public_key::*, refresh::*, request::*, response::*, secret_key::*,
    signature::*, xnode::*,
};

/// These tests fix the JSON form of the byte-array types, so that a change
/// of the hex library can not change the wire format.
#[cfg(test)]
mod wire_format_tests {
    use super::*;

    #[test]
    fn byte_arrays_serialize_as_lowercase_hex() {
        let hash = Hash([0xab; 32]);
        assert_eq!(
            serde_json::to_string(&hash).unwrap(),
            format!("\"{}\"", "ab".repeat(32))
        );

        let policy = PolicyId([0x0f; 28]);
        assert_eq!(
            serde_json::to_string(&policy).unwrap(),
            format!("\"{}\"", "0f".repeat(28))
        );
    }

    #[test]
    fn byte_arrays_round_trip() {
        let key = PublicKey([7u8; 32]);
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(serde_json::from_str::<PublicKey>(&json).unwrap(), key);
    }

    #[test]
    fn byte_arrays_accept_uppercase_hex() {
        let json = format!("\"{}\"", "AB".repeat(32));
        assert_eq!(
            serde_json::from_str::<Hash>(&json).unwrap(),
            Hash([0xab; 32])
        );
    }

    #[test]
    fn byte_arrays_reject_the_wrong_length() {
        let short = format!("\"{}\"", "ab".repeat(31));
        let long = format!("\"{}\"", "ab".repeat(33));
        assert!(serde_json::from_str::<Hash>(&short).is_err());
        assert!(serde_json::from_str::<Hash>(&long).is_err());
        assert!(serde_json::from_str::<Hash>("\"zz\"").is_err());
    }
}
