pub mod args;
pub mod db;
pub mod fetch;

use db::create_table;
use sqlx::postgres::PgPoolOptions;

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = args::parse().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(args.database_conn)
        .connect(args.database_url.as_str())
        .await?;
    let markets = args.clone().markets;
    // creating table for each market
    for market in markets.iter() {
        let mut conn = pool.acquire().await.unwrap();
        create_table(&mut conn, market.as_str(), &args.currencies).await?;
    }
    Ok(())
}
