use mugraph_core::{error::Error, types::*};
use mugraph_node::{database::Database, v0::transaction_v0};
use reqwest::blocking::Client;

#[derive(Debug)]
pub enum NodeTarget {
  Local,
  Remote(String),
}

#[derive(Debug)]
pub struct Node {
  client: Client,
  target: NodeTarget,
}

impl Node {
  pub fn new(target: NodeTarget) -> Result<Self, Error> {
    let client = Client::new();
    Ok(Self { target, client })
  }

  pub fn execute_transaction_v0(
    &mut self,
    transaction: &Transaction,
    keypair: Keypair,
    database: &mut Database,
  ) -> Result<V0Response, Error> {
    match &self.target {
      NodeTarget::Remote(_) => self.remote(transaction),
      NodeTarget::Local => self.local(transaction, keypair, database),
    }
  }

  fn remote(
    &mut self,
    transaction: &Transaction,
  ) -> Result<V0Response, Error> {
    let target_endpoint = match &self.target {
      NodeTarget::Remote(url) => format!("{}/v0/rpc", url),
      NodeTarget::Local => return Err(Error::Other),
    };

    let request = Request::V0(V0Request::Transaction(transaction.clone()));

    let response = self
      .client
      .post(&target_endpoint)
      .json(&request)
      .send()
      .map_err(|err| Error::ServerError {
        reason: err.to_string(),
      })?;

    let response_text = response.text().map_err(|err| Error::ServerError {
      reason: err.to_string(),
    })?;
    let v0_response: V0Response = serde_json::from_str(&response_text).map_err(|_| {
      if response_text.contains("Atom has already been spent") {
        return Error::AlreadySpent {
          signature: transaction.signatures[0],
        };
      }
      Error::ServerError {
        reason: response_text,
      }
    })?;

    Ok(v0_response)
  }

  fn local(
    &mut self,
    transaction: &Transaction,
    keypair: Keypair,
    database: &mut Database,
  ) -> Result<V0Response, Error> {
    transaction_v0(transaction, keypair, database)
  }
}
