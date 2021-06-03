use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use std::collections::HashMap;

pub fn get_table_name(market: &str) -> String {
    format!("price_{}", market)
}

pub async fn create_table(
    conn: &mut PoolConnection<Postgres>,
    market: &str,
    currencies: &Vec<String>,
) -> Result<()> {
    let tbl = get_table_name(market);
    let mut currency_fields: Vec<String> = vec![];
    for currency in currencies.iter() {
        currency_fields.push(format!("{} numeric(20,10) not null,", currency))
    }
    let sql = format!(
        "create table if not exists {} (
        ts timestamp without time zone not null, {} primary key (ts))",
        tbl,
        currency_fields.join(", ")
    );
    sqlx::query(sql.as_str()).execute(conn).await?;
    info!("table {} created", tbl);
    Ok(())
}

pub async fn insert(
    timestamp: DateTime<Utc>,
    conn: &mut PoolConnection<Postgres>,
    market: &str,
    prices: &HashMap<String, f64>,
) -> Result<()> {
    let mut fields: Vec<String> = vec![];
    let mut values: Vec<String> = vec![];
    for (field, value) in prices.iter() {
         fields.push(field.clone());
         values.push(format!("{}",value));
     }
    let sql = format!(
        "INSERT INTO {} (ts,{}) VALUES (?,{}) ON CONFLICT DO NOTHING",
        get_table_name(market),
        fields.join(", "),
        values.join(", ")
    );
    sqlx::query(sql.as_str()).bind(timestamp).execute(conn).await?;
    Ok(())
}
