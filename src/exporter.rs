use super::{db, fetch};
use crate::{Currencies, Markets};
use anyhow::Result;
use async_std::task;
use chrono::prelude::*;
use chrono::{Datelike, Duration, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use tracing::info;

pub async fn init(
    conn: &mut PoolConnection<Postgres>,
    markets: &Markets,
    currencies: &Currencies,
) -> Result<()> {
    for market in markets.iter() {
        db::create_table(conn, market.name.as_str(), currencies).await?;
    }
    Ok(())
}

pub async fn update_history(
    conn: &mut PoolConnection<Postgres>,
    markets: &Markets,
    currencies: &Currencies,
    no_gaps: bool,
) -> Result<()> {
    // creating table for each market
    // and fetchign missing history
    for market in markets.iter() {
        let span = std::time::Instant::now();
        let mut days = 0;
        let now = Utc::now();
        let start = Utc.ymd(now.year(), now.month(), now.day());
        println!("Market {}: updating history since {:?}", &market.name, market.earliest);
        loop {
            let dt = start + Duration::days(days);
            let earliest = Date::<Utc>::from_utc(market.earliest, Utc);
            if dt < earliest {
                break;
            }
            let timestamp = dt.and_hms(0, 0, 0);

            let y = dt.year();
            let m = dt.month();
            let d = dt.day();
            if db::has_price(conn, timestamp, &market.name).await {
                days = days - 1;
                continue;
            }
            info!(
                "missing price for {}: {}-{:02}-{:02}",
                market.name.as_str(),
                y,
                m,
                d
            );
            match fetch::history(market.name.as_str(), y, m, d, currencies) {
                Ok(prices) => {
                    db::insert(conn, timestamp, &market.name, &prices).await?;
                },
                Err(_) => {
                    if no_gaps {
                        let gaps_map = currencies.as_map();
                        db::insert(conn, timestamp, &market.name, &gaps_map).await?;
                    }
                },
            };
            task::sleep(std::time::Duration::from_secs(1)).await;
            days = days - 1;
        }
        info!("Indexing of {} market took {:?}", &market.name, span.elapsed());
    }
    Ok(())
}
