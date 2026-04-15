use std::time::Duration;

use mugraph_core::types::{
    BlindSignature, PublicKey, Refresh, Request, Response,
};
use reqwest::Url;

#[derive(Clone)]
pub struct NodeClient {
    http: reqwest::Client,
    rpc_url: Url,
    health_url: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum NodeClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("node error: {reason}")]
    Node { reason: String },
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl NodeClient {
    pub fn new(base: &Url) -> Result<Self, NodeClientError> {
        let mut rpc_url = base.clone();
        rpc_url.set_path("/rpc");

        let mut health_url = base.clone();
        health_url.set_path("/health");

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(3))
            .build()?;

        Ok(Self {
            http,
            rpc_url,
            health_url,
        })
    }

    pub async fn health(&self) -> Result<(), NodeClientError> {
        let res = self.http.get(self.health_url.clone()).send().await?;
        if !res.status().is_success() {
            return Err(NodeClientError::Node {
                reason: format!("health check failed with {}", res.status()),
            });
        }
        Ok(())
    }

    pub async fn info(
        &self,
    ) -> Result<(PublicKey, Option<String>), NodeClientError> {
        match self.rpc(&Request::Info).await? {
            Response::Info {
                delegate_pk,
                cardano_script_address,
            } => Ok((delegate_pk, cardano_script_address)),
            Response::Error { reason } => Err(NodeClientError::Node { reason }),
            other => {
                Err(NodeClientError::UnexpectedResponse(format!("{other:?}")))
            }
        }
    }

    pub async fn refresh(
        &self,
        refresh: &Refresh,
    ) -> Result<Vec<BlindSignature>, NodeClientError> {
        match self.rpc(&Request::Refresh(refresh.clone())).await? {
            Response::Transaction { outputs } => Ok(outputs),
            Response::Error { reason } => Err(NodeClientError::Node { reason }),
            other => {
                Err(NodeClientError::UnexpectedResponse(format!("{other:?}")))
            }
        }
    }

    async fn rpc(
        &self,
        request: &Request,
    ) -> Result<Response, NodeClientError> {
        let res = self
            .http
            .post(self.rpc_url.clone())
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builds_urls_from_base() {
        let base = Url::parse("http://localhost:3000").unwrap();
        let client = NodeClient::new(&base).unwrap();
        assert_eq!(client.rpc_url.as_str(), "http://localhost:3000/rpc");
        assert_eq!(client.health_url.as_str(), "http://localhost:3000/health");
    }

    #[test]
    fn new_strips_existing_path() {
        let base = Url::parse("http://localhost:3000/old/path").unwrap();
        let client = NodeClient::new(&base).unwrap();
        assert_eq!(client.rpc_url.path(), "/rpc");
        assert_eq!(client.health_url.path(), "/health");
    }

    #[test]
    fn error_display_includes_reason() {
        let err = NodeClientError::Node {
            reason: "test error".to_string(),
        };
        assert!(err.to_string().contains("test error"));
    }
}
