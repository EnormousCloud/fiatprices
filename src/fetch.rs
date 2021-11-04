use crate::{Currencies, Markets};
use anyhow::Result;
use cached::proc_macro::cached;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::warn;
use ureq::{Agent, AgentBuilder};

#[cached(time = 300)]
pub fn cached_get(url: String) -> String {
    let agent: Agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .build();
    let response = match agent.get(url.as_str()).call() {
        Ok(x) => x.into_string().unwrap_or("{}".to_owned()),
        Err(_) => "{}".to_owned(),
    };
    response
}

pub fn current(markets: &Markets, currencies: &Currencies) -> String {
    cached_get(format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        markets.as_vec().join("%2C"),
        currencies.as_vec().join("%2C")
    ))
}

#[derive(Clone, Debug, Deserialize)]
pub struct MarketData {
    current_price: HashMap<String, f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HistoryResponse {
    market_data: MarketData,
}

pub fn history(
    market: &str,
    y: i32,
    m: u32,
    d: u32,
    currencies: &Currencies,
) -> Result<HashMap<String, f64>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/{}/history?date={:02}-{:02}-{}",
        market, d, m, y,
    );
    let agent: Agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .build();
    let raw = agent.get(url.as_str()).call()?.into_string()?;
    let result = str::replace(raw.as_str(), "null", "0");
    let response: HistoryResponse = match serde_json::from_str(result.as_str()) {
        Ok(x) => x,
        Err(e) => {
            warn!("LAST RESPONSE: {}", result);
            warn!("ERROR: {}", e);
            return Err(anyhow::Error::from(e));
        }
    };
    let mut out: HashMap<String, f64> = HashMap::new();
    for currency in currencies.iter() {
        if let Some(val) = response.market_data.current_price.get(currency) {
            out.insert(currency.clone(), *val);
        }
    }
    Ok(out.clone())
}
