use std::{collections::HashSet, fs, path::Path};

use mugraph_core::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRegistry {
    pub peers: Vec<TrustedPeer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedPeer {
    pub node_id: String,
    pub endpoint: String,
    pub auth_alg: String,
    pub kid: String,
    pub public_key_hex: String,
    #[serde(default)]
    pub revoked: bool,
}

impl PeerRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let contents =
            fs::read_to_string(path).map_err(|e| Error::InvalidInput {
                reason: format!(
                    "failed to read peer registry {}: {e}",
                    path.display()
                ),
            })?;

        serde_json::from_str(&contents).map_err(|e| Error::InvalidInput {
            reason: format!(
                "invalid peer registry JSON {}: {e}",
                path.display()
            ),
        })
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut unique_node_kid: HashSet<(&str, &str)> = HashSet::new();

        for peer in &self.peers {
            if peer.auth_alg != "Ed25519" {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "peer {} uses unsupported auth_alg {}, expected Ed25519",
                        peer.node_id, peer.auth_alg
                    ),
                });
            }

            if reqwest::Url::parse(&peer.endpoint).is_err() {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "peer {} endpoint is not a valid URL",
                        peer.node_id
                    ),
                });
            }

            let key_bytes =
                muhex::decode(&peer.public_key_hex).map_err(|e| {
                    Error::InvalidInput {
                        reason: format!(
                            "peer {} public_key_hex is not valid hex: {e}",
                            peer.node_id
                        ),
                    }
                })?;

            if key_bytes.len() != 32 {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "peer {} public_key_hex must be 32 bytes (got {})",
                        peer.node_id,
                        key_bytes.len()
                    ),
                });
            }

            if !unique_node_kid.insert((&peer.node_id, &peer.kid)) {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "duplicate peer key mapping for node_id={} kid={}",
                        peer.node_id, peer.kid
                    ),
                });
            }
        }

        Ok(())
    }
}
