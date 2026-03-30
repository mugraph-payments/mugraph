use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use mugraph_core::{error::Error, types::XNodeEnvelope};
use serde::Serialize;

use super::protocol_reject;
use crate::routes::Context;

pub(super) const MAX_CLOCK_SKEW_SECS: i64 = 300;
pub(super) const MAX_COMMAND_EXPIRY_HORIZON_SECS: i64 = 900;
pub(super) const AUTH_DOMAIN_SEP: &[u8] = b"mugraph_xnode_auth_v1";

pub(super) fn validate_freshness(
    sent_at: &str,
    expires_at: Option<&str>,
    now: i64,
) -> Result<(), Error> {
    let sent_at = DateTime::parse_from_rfc3339(sent_at)
        .map_err(|e| {
            protocol_reject(
                "SCHEMA_VALIDATION_FAILED",
                format!("invalid sent_at timestamp format: {e}"),
            )
        })?
        .with_timezone(&Utc)
        .timestamp();

    let expires_at = expires_at.ok_or_else(|| {
        protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            "expires_at is required for command envelopes",
        )
    })?;
    let expires_at = DateTime::parse_from_rfc3339(expires_at)
        .map_err(|e| {
            protocol_reject(
                "SCHEMA_VALIDATION_FAILED",
                format!("invalid expires_at timestamp format: {e}"),
            )
        })?
        .with_timezone(&Utc)
        .timestamp();

    if expires_at <= sent_at {
        return Err(protocol_reject(
            "REPLAY_DETECTED",
            "expired command envelope",
        ));
    }

    if (now - sent_at).abs() > MAX_CLOCK_SKEW_SECS {
        return Err(protocol_reject(
            "REPLAY_DETECTED",
            "sent_at outside allowed clock skew",
        ));
    }

    if expires_at < now {
        return Err(protocol_reject(
            "REPLAY_DETECTED",
            "command envelope already expired",
        ));
    }

    if expires_at - sent_at > MAX_COMMAND_EXPIRY_HORIZON_SECS {
        return Err(protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            "command expiry horizon exceeds policy",
        ));
    }

    Ok(())
}

pub(super) fn validate_query_freshness(
    sent_at: &str,
    now: i64,
) -> Result<(), Error> {
    let sent_at = DateTime::parse_from_rfc3339(sent_at)
        .map_err(|e| {
            protocol_reject(
                "SCHEMA_VALIDATION_FAILED",
                format!("invalid sent_at timestamp format: {e}"),
            )
        })?
        .with_timezone(&Utc)
        .timestamp();

    if (now - sent_at).abs() > MAX_CLOCK_SKEW_SECS {
        return Err(protocol_reject(
            "REPLAY_DETECTED",
            "sent_at outside allowed clock skew",
        ));
    }

    Ok(())
}

pub(super) fn validate_destination_binding<T>(
    request: &XNodeEnvelope<T>,
    local_node_id: &str,
) -> Result<(), Error> {
    if request.origin_node_id == request.destination_node_id {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "origin and destination nodes must differ",
        ));
    }

    if !request.origin_node_id.starts_with("node://")
        || !request.destination_node_id.starts_with("node://")
    {
        return Err(protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            "origin/destination node ids must use node:// scheme",
        ));
    }

    if request.destination_node_id != local_node_id {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "destination_node_id does not match local node id",
        ));
    }

    Ok(())
}

fn load_peer_registry_for_auth(
    ctx: &Context,
) -> Result<std::borrow::Cow<'_, crate::peer_registry::PeerRegistry>, Error> {
    if let Some(path) = ctx.config.xnode_peer_registry_file() {
        let registry = crate::peer_registry::PeerRegistry::load(&path)?;
        registry.validate()?;
        return Ok(std::borrow::Cow::Owned(registry));
    }

    let Some(registry) = ctx.peer_registry.as_ref() else {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "xnode peer registry is required for cross-node command auth",
        ));
    };

    Ok(std::borrow::Cow::Borrowed(registry.as_ref()))
}

pub(super) fn validate_auth_signature<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    ctx: &Context,
) -> Result<(), Error> {
    if request.auth.alg != "Ed25519" {
        return Err(protocol_reject("AUTHZ_DENIED", "unsupported auth.alg"));
    }

    let registry = load_peer_registry_for_auth(ctx)?;

    let peer = registry
        .peers
        .iter()
        .find(|p| {
            !p.revoked
                && p.node_id == request.origin_node_id
                && p.kid == request.auth.kid
                && p.auth_alg == request.auth.alg
        })
        .ok_or_else(|| {
            protocol_reject("UNKNOWN_KEY_ID", "untrusted origin node or key id")
        })?;

    let pubkey = muhex::decode(&peer.public_key_hex).map_err(|e| {
        protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            format!("invalid trusted peer public key hex: {e}"),
        )
    })?;
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey.as_slice().try_into().map_err(|_| {
            protocol_reject(
                "SCHEMA_VALIDATION_FAILED",
                "trusted peer public key must be 32 bytes",
            )
        })?,
    )
    .map_err(|e| {
        protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            format!("invalid trusted peer public key: {e}"),
        )
    })?;

    let sig_bytes = muhex::decode(&request.auth.sig).map_err(|e| {
        protocol_reject(
            "INVALID_SIGNATURE",
            format!("invalid auth signature hex: {e}"),
        )
    })?;
    let sig = Signature::try_from(sig_bytes.as_slice()).map_err(|e| {
        protocol_reject(
            "INVALID_SIGNATURE",
            format!("invalid auth signature bytes: {e}"),
        )
    })?;

    let payload = canonical_auth_payload(request)?;
    verifying_key.verify(&payload, &sig).map_err(|e| {
        protocol_reject(
            "INVALID_SIGNATURE",
            format!("invalid auth signature: {e}"),
        )
    })?;

    Ok(())
}

pub(super) fn canonical_auth_payload<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
) -> Result<Vec<u8>, Error> {
    let mut canonical = request.clone();
    canonical.auth.sig.clear();

    let body = serde_json::to_vec(&canonical)?;
    let mut payload = Vec::with_capacity(AUTH_DOMAIN_SEP.len() + body.len());
    payload.extend_from_slice(AUTH_DOMAIN_SEP);
    payload.extend_from_slice(&body);
    Ok(payload)
}
