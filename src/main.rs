pub mod args;
pub mod db;
pub mod fetch;

use chrono::{Duration, Utc};

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = match args::parse() {
        Ok(x) => x,
        Err(e) => {
            panic!("Args parsing error: {}", e);
        }
    };

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(args.database_conn)
        .connect_timeout(std::time::Duration::from_secs(3))
        .connect(args.database_url.as_str())
        .await
        .unwrap();
    let mut conn = match pool.acquire().await {
        Ok(x) => x,
        Err(e) => {
            panic!(
                "Database connection failure {} url={}",
                e, args.database_url
            );
        }
    };

    let markets = args.clone().markets;
    // creating table for each market
    for market in markets.iter() {
        db::create_table(&mut conn, market.as_str(), &args.currencies).await?;
        let mut days = 0;
        loop {
            let dt = Utc::now() + Duration::days(days);
            let stop: bool = match fetch::history(market.as_str(), 2013, 12, 31, &args.currencies) {
                Ok(prices) => {
                    db::insert(&mut conn,dt, market, &prices).await?;
                    false
                },
                Err(_) => { true }
            };
            if stop {
                break;
            }
            days = days - 1;
        }
    }

    // if there is no table, 
    // start day by day since today till the error
    // let h = fetch::history("bitcoin", 2013, 12, 31, &args.currencies)?;
    // println!("map {:?}", h);

    Ok(())
}
// fetch day by day
