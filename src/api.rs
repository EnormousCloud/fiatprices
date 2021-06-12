use crate::{db, fetch, State};
use chrono::prelude::*;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};
use tide::{Body, Request, Response, Result};

pub async fn health(_: Request<State>) -> Result {
    let mut m: HashMap<&str, String> = HashMap::new();
    m.insert("app", "fiatprices".to_owned());

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&m)?);
    Ok(res)
}

pub async fn current(req: Request<State>) -> Result {
    let mut res = Response::new(200);
    let val = fetch::current(&req.state().markets, &req.state().currencies);
    res.set_body(val.as_str());
    Ok(res)
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct HistoryResponse {
    pub markets: HashMap<String, BTreeMap<String, f64>>,
}
impl HistoryResponse {
    pub fn new(market: &str, prices: BTreeMap<String, f64>) -> Self {
        let mut m: HashMap<String, BTreeMap<String, f64>> = HashMap::new();
        m.insert(market.to_owned(), prices);
        Self { markets: m }
    }
}

pub async fn history(req: Request<State>) -> Result {
    let market = req.param("market").unwrap_or("none");
    let iso8601 = req.param("date").unwrap_or("none");
    let dt = match NaiveDate::parse_from_str(iso8601, "%Y-%m-%d") {
        Ok(x) => x,
        Err(e) => {
            let mut res = Response::new(400);
            res.set_body(format!("{:?}", e));
            return Ok(res);
        }
    };
    let y = dt.year();
    let m = dt.month();
    let d = dt.day();
    let tm: DateTime<Utc> = Utc.ymd(y, m, d).and_hms(0, 0, 0);
    log::info!("market={}, y={}, m={}, d={} dt={}", market, y, m, d, dt);

    let db_pool = req.state().db_pool.clone();
    let mut conn = db_pool.acquire().await?;
    let prices = db::get_prices(&mut conn, tm, market, &req.state().currencies).await;

    let mut res = Response::new(200);
    let response = HistoryResponse::new(market, prices);
    res.set_body(serde_json::to_string(&response)?);
    Ok(res)
}
