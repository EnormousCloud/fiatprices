use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

pub fn current(markets: &Vec<String>, currencies: &Vec<String>) -> Result<()> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        markets.join("%2C"),
        currencies.join("%2C")
    );
    let agent: Agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .build();
    let response = agent.get(url.as_str()).call()?.into_string()?;
    log::warn!("current {}", response);
    Ok(())
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
    y: u32,
    m: u32,
    d: u32,
    currencies: &Vec<String>,
) -> Result<HashMap<String, f64>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/{}/history?date={}-{}-{}",
        market, d, m, y,
    );
    let agent: Agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .build();
    let result = agent.get(url.as_str()).call()?.into_string()?;
    let response: HistoryResponse = match serde_json::from_str(result.as_str()) {
        Ok(x) => x,
        Err(e) => {
            // log::warn!("RESPONSE: {}", result);
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
