use super::{db, fetch};
use std::collections::HashMap;
use anyhow::Result;
use async_std::task;
use chrono::prelude::*;
use chrono::{Datelike, Duration, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use crate::{Markets, Currencies};

pub async fn init(
    conn: &mut PoolConnection<Postgres>,
    markets: &Markets,
    currencies: &Currencies,
) -> Result<()> {
    for market in markets.iter() {
        db::create_table(conn, market.as_str(), currencies).await?;
    }
    Ok(())
}

pub async fn update_history(
    conn: &mut PoolConnection<Postgres>,
    markets: &Markets,
    currencies: &Currencies,
) -> Result<()> {
    let mut earliest_map : HashMap<&str, DateTime<Utc>> = HashMap::new();
    earliest_map.insert("ethereum", 
    Utc.ymd(2015, 8, 1).and_hms(0, 0, 0));
    earliest_map.insert("bitcoin", 
    Utc.ymd(2013, 5, 1).and_hms(0, 0, 0));

    // creating table for each market
    // and fetchign missing history
    for market in markets.iter() {
        let mut days = 0;
        let now = Utc::now();
        let start = Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0);
        let earliest = earliest_map.get(market.as_str()).unwrap();
        
        loop {
            let dt = start + Duration::days(days);
            if dt < *earliest {
                break;
            }

            let y = dt.year();
            let m = dt.month();
            let d = dt.day();
            if db::has_price(conn, dt, market).await {
                days = days - 1;
                continue;
            }
            log::info!(
                "missing price for {}: {}-{:02}-{:02}",
                market.as_str(),
                y,
                m,
                d
            );
            if let Ok (prices) = fetch::history(market.as_str(), y, m, d, currencies) {
                db::insert(conn, dt, market, &prices).await?;
            };
            task::sleep(std::time::Duration::from_secs(1)).await;
            days = days - 1;
        }
    }
    Ok(())
}
