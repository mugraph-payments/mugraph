use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

pub(super) const PROVIDER_MAX_RETRIES: usize = 3;
pub(super) const PROVIDER_BACKOFF_MS: u64 = 200;
pub(super) const ADDRESS_UTXO_PAGE_SIZE: usize = 100;

/// Protocol parameters for fee calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParams {
    pub min_fee_a: u64,
    pub min_fee_b: u64,
    pub max_tx_size: u64,
    pub max_val_size: u64,
    pub key_deposit: u64,
    pub pool_deposit: u64,
    pub price_mem: f64,
    pub price_step: f64,
    pub max_tx_ex_mem: u64,
    pub max_tx_ex_steps: u64,
    pub coins_per_utxo_byte: u64,
}

pub(super) async fn send_with_retry<F>(
    make: F,
    context: &str,
) -> Result<reqwest::Response>
where
    F: Fn() -> reqwest::RequestBuilder,
{
    let mut delay = PROVIDER_BACKOFF_MS;
    for attempt in 1..=PROVIDER_MAX_RETRIES {
        let resp_result = make().send().await;
        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() || status.as_u16() == 404 {
                    return Ok(resp);
                }

                if attempt == PROVIDER_MAX_RETRIES
                    || !(status.is_server_error() || status.as_u16() == 429)
                {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(color_eyre::eyre::eyre!(
                        "{} (status {}): {}",
                        context,
                        status,
                        text
                    ));
                }
            }
            Err(e) => {
                if attempt == PROVIDER_MAX_RETRIES {
                    return Err(color_eyre::eyre::eyre!(
                        "{} (network error after {} attempts): {}",
                        context,
                        attempt,
                        e
                    ));
                }
            }
        }

        sleep(Duration::from_millis(delay)).await;
        delay *= 2;
    }

    Err(color_eyre::eyre::eyre!("{}: exceeded max retries", context))
}

pub(super) fn parse_required<T>(field: &str, value: &str) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    value.parse::<T>().map_err(|e| {
        color_eyre::eyre::eyre!("invalid protocol param {field}={value}: {e}")
    })
}

pub(super) fn with_pagination(
    base_url: &str,
    page: usize,
    count: usize,
) -> String {
    format!("{base_url}?page={page}&count={count}")
}
