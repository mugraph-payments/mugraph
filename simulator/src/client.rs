use std::time::Duration;

use color_eyre::eyre::{Result, WrapErr, eyre};
use mugraph_core::types::{AssetName, Note, PolicyId, PublicKey, Refresh, Request, Response};
use reqwest::Url;

#[derive(Clone)]
pub struct NodeClient {
    http: reqwest::Client,
    rpc_url: Url,
    health_url: Url,
}

impl NodeClient {
    pub fn new(base: &Url) -> Result<Self> {
        let mut rpc_url = base.clone();
        rpc_url.set_path("/rpc");

        let mut health_url = base.clone();
        health_url.set_path("/health");

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .wrap_err("build http client")?;

        Ok(Self {
            http,
            rpc_url,
            health_url,
        })
    }

    pub async fn health(&self) -> Result<()> {
        let res = self.http.get(self.health_url.clone()).send().await?;
        if !res.status().is_success() {
            return Err(eyre!("health check failed with {}", res.status()));
        }
        Ok(())
    }

    pub async fn public_key(&self) -> Result<PublicKey> {
        match self.rpc(&Request::Info).await? {
            Response::Info { delegate_pk, .. } => Ok(delegate_pk),
            Response::Error { reason } => Err(eyre!("public_key failed: {}", reason)),
            other => Err(eyre!("unexpected response for public_key: {:?}", other)),
        }
    }

    pub async fn rpc(&self, request: &Request) -> Result<Response> {
        let res = self
            .http
            .post(self.rpc_url.clone())
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    pub async fn emit(&self, policy_id: PolicyId, asset_name: AssetName, amount: u64) -> Result<Note> {
        match self
            .rpc(&Request::Emit {
                policy_id,
                asset_name,
                amount,
            })
            .await?
        {
            Response::Emit(note) => Ok(*note),
            Response::Error { reason } => Err(eyre!("emit failed: {}", reason)),
            other => Err(eyre!("unexpected response for emit: {:?}", other)),
        }
    }

    pub async fn refresh(&self, refresh: &Refresh) -> Result<Vec<mugraph_core::types::BlindSignature>> {
        match self.rpc(&Request::Refresh(refresh.clone())).await? {
            Response::Transaction { outputs } => Ok(outputs),
            Response::Error { reason } => Err(eyre!("refresh failed: {}", reason)),
            other => Err(eyre!("unexpected response for refresh: {:?}", other)),
        }
    }
}
