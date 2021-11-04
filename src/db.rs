use crate::Currencies;
use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use sqlx::{Postgres, Row};
use std::collections::{BTreeMap, HashMap};

pub fn get_table_name(market: &str) -> String {
    format!("price_{}", market)
}

pub async fn create_table(
    conn: &mut PoolConnection<Postgres>,
    market: &str,
    currencies: &Currencies,
) -> Result<()> {
    let tbl = get_table_name(market);
    let mut currency_fields: Vec<String> = vec![];
    for currency in currencies.iter() {
        currency_fields.push(format!("{} numeric(20,10) not null", currency))
    }
    let sql = format!(
        "create table if not exists {} (
        ts timestamptz not null, {}, primary key (ts))",
        tbl,
        currency_fields.join(", ")
    );
    if let Err(e) = sqlx::query(&sql).execute(conn).await {
        panic!("sql create error {}", e);
    };
    Ok(())
}

pub async fn get_prices(
    conn: &mut PoolConnection<Postgres>,
    timestamp: DateTime<Utc>,
    market: &str,
    currencies: &Currencies,
) -> BTreeMap<String, f64> {
    let sql = format!(
        "SELECT {} FROM {} WHERE ts >= $1 LIMIT 1",
        currencies.as_vec().join(","),
        get_table_name(market),
    );
    let row = match sqlx::query(&sql).bind(timestamp).fetch_one(conn).await {
        Ok(x) => x,
        Err(e) => {
            panic!("sql prices has error {}", e);
        }
    };
    let mut m = BTreeMap::new();
    for currency in currencies.iter() {
        let num: BigDecimal = row.get(currency.as_str());
        let val: f64 = num.to_string().parse().unwrap_or(0.0);
        m.insert(currency.to_string(), val);
    }
    m
}

pub async fn has_price(
    conn: &mut PoolConnection<Postgres>,
    timestamp: DateTime<Utc>,
    market: &str,
) -> bool {
    let sql = format!(
        "SELECT ts FROM {} WHERE ts = $1 LIMIT 1",
        get_table_name(market),
    );
    let tsvec: Vec<DateTime<Utc>> = match sqlx::query_scalar(&sql)
        .bind(timestamp)
        .fetch_all(conn)
        .await
    {
        Ok(x) => x,
        Err(e) => {
            panic!("sql has error {}", e);
        }
    };
    tsvec.len() > 0
}

pub async fn insert(
    conn: &mut PoolConnection<Postgres>,
    timestamp: DateTime<Utc>,
    market: &str,
    prices: &HashMap<String, f64>,
) -> Result<()> {
    let mut fields: Vec<String> = vec![];
    let mut values: Vec<String> = vec![];
    for (field, value) in prices.iter() {
        fields.push(field.clone());
        values.push(format!("{}", value));
    }
    let sql = format!(
        "INSERT INTO {} (ts,{}) VALUES ($1,{}) ON CONFLICT DO NOTHING",
        get_table_name(market),
        fields.join(", "),
        values.join(", ")
    );
    if let Err(e) = sqlx::query(&sql).bind(timestamp).execute(conn).await {
        panic!("sql insert error {}", e);
    };
    Ok(())
}
