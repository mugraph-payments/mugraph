use blake2::{Blake2b, Digest, digest::consts::U32};
use pallas_addresses::Address;
use pallas_primitives::babbage::PseudoDatumOption;
use pallas_traverse::MultiEraTx;

#[derive(Debug)]
pub struct ParsedOutput {
    pub address: String,
    pub lovelace: u64,
    pub datum_hash: Option<String>,
    pub inline_datum_cbor: Option<String>,
}

#[derive(Debug)]
pub struct ParsedTx {
    pub tx_hash: String,
    pub inputs: Vec<(String, u16)>,
    pub outputs: Vec<ParsedOutput>,
    pub fee: u64,
}

impl ParsedTx {
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, String> {
        let tx =
            MultiEraTx::decode(bytes).map_err(|e| format!("decode: {e}"))?;

        let tx_hash = hex::encode(tx.hash());

        let inputs = tx
            .inputs()
            .into_iter()
            .map(|i| (hex::encode(i.hash()), i.index() as u16))
            .collect();

        let mut outputs = Vec::new();
        for out in tx.outputs() {
            let address =
                out.address().map_err(|e| format!("output address: {e}"))?;
            let address_str = match address {
                Address::Byron(_) => "byron-not-supported".to_string(),
                a => {
                    a.to_bech32().map_err(|e| format!("address bech32: {e}"))?
                }
            };
            let (datum_hash, inline_datum_cbor) = match out.datum() {
                Some(PseudoDatumOption::Hash(h)) => {
                    (Some(hex::encode(h)), None)
                }
                Some(PseudoDatumOption::Data(d)) => {
                    let bytes = d.raw_cbor().to_vec();
                    let hash = blake2b_256(&bytes);
                    (Some(hex::encode(hash)), Some(hex::encode(&bytes)))
                }
                None => (None, None),
            };
            outputs.push(ParsedOutput {
                address: address_str,
                lovelace: out.value().coin(),
                datum_hash,
                inline_datum_cbor,
            });
        }

        let fee = tx.fee().unwrap_or(0);

        Ok(Self {
            tx_hash,
            inputs,
            outputs,
            fee,
        })
    }
}

fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = <Blake2b<U32>>::new();
    hasher.update(data);
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_garbage_bytes() {
        let err = ParsedTx::from_cbor(&[0xff, 0xff, 0xff]).unwrap_err();
        assert!(err.contains("decode"));
    }
}
