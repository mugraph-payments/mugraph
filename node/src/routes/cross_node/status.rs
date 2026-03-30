use ed25519_dalek::Signer;
use mugraph_core::{error::Error, types::XNodeEnvelope};
use serde::Serialize;

use super::{auth::canonical_auth_payload, protocol_reject};
use crate::{database::CARDANO_WALLET, routes::Context};

pub(super) fn sign_status_response<T: Serialize + Clone>(
    envelope: &mut XNodeEnvelope<T>,
    ctx: &Context,
) -> Result<(), Error> {
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(CARDANO_WALLET)?;
    let wallet = table.get("wallet")?.map(|v| v.value()).ok_or_else(|| {
        protocol_reject(
            "AUTHZ_DENIED",
            "wallet not initialized; cannot sign status response",
        )
    })?;

    let sk_bytes: [u8; 32] =
        wallet.payment_sk.as_slice().try_into().map_err(|_| {
            protocol_reject(
                "SCHEMA_VALIDATION_FAILED",
                "wallet payment signing key must be 32 bytes",
            )
        })?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
    let payload = canonical_auth_payload(envelope)?;
    let sig = signing_key.sign(&payload);
    envelope.auth.sig = muhex::encode(sig.to_bytes());
    Ok(())
}
