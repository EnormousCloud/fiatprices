pub mod api;
pub mod args;
pub mod db;
pub mod exporter;
pub mod fetch;
pub mod telemetry;
pub mod metrics;

use tracing::info;
use std::collections::HashMap;
use chrono::{Datelike, NaiveDate, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Market {
    pub name: String,
    pub earliest: NaiveDate,
}

impl Market {
    pub fn new(src: &str) -> Self {
        let parts: Vec<&str> = src.split(":").collect();
        let now = Utc::now();
        let start = NaiveDate::from_ymd(now.year(), 1, 1);
        let (name, earliest) = if parts.len() == 1 {
            (src.to_owned(), start)
        } else {
            let dt = match NaiveDate::parse_from_str(parts[1], "%Y-%m-%d") {
                Ok(x) => x,
                Err(_) => start,
            };
            (parts[0].to_owned(), dt)
        };
        Self {
            name,
            earliest,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Markets(Vec<Market>);
impl std::str::FromStr for Markets {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Markets(s.split(",").map(|x| Market::new(x.trim())).collect()))
    }
}
impl Markets {
    pub fn as_vec(&self) -> Vec<String> {
        self.0.iter().map(|x| x.name.clone()).collect()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Market> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Currencies(Vec<String>);
impl std::str::FromStr for Currencies {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Currencies(
            s.split(",").map(|x| x.trim().to_owned()).collect(),
        ))
    }
}
impl Currencies {
    pub fn as_vec(&self) -> Vec<String> {
        self.0.clone()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, std::string::String> {
        self.0.iter()
    }
    pub fn as_map(&self) -> HashMap<String, f64> {
        self.0.iter().map(|s| (s.clone(), -1f64))
        .into_iter()
        .collect()
    }
}

#[derive(Clone)]
pub struct State {
    pub db_pool: sqlx::Pool<sqlx::postgres::Postgres>,
    pub markets: Markets,
    pub currencies: Currencies,
}

use tide::{Middleware, Next, Request};

// This is an example of middleware that keeps its own state and could
// be provided as a third party crate
#[derive(Default)]
struct LogMiddleware {}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LogMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        // let path = req.url().path().to_owned();
        // let method = req.method().to_string();
        // println!("method={} path={}", method, path);
        Ok(next.run(req).await)
    }
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
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

    exporter::init(&mut conn, &args.markets, &args.currencies).await?;
    if args.index > 0 {
        let no_gaps = args.index > 1;
        exporter::update_history(&mut conn, &args.markets, &args.currencies, no_gaps).await?;
    }
    if args.server > 0 {
        let state = State {
            db_pool: pool,
            markets: args.markets.clone(),
            currencies: args.currencies.clone(),
        };
        info!("Starting HTTP server {}", &args.addr);
        let mut app = tide::with_state(state);
        app.with(LogMiddleware {});
        app.with(telemetry::TraceMiddleware::new());
        app.at("/metrics").get(api::metrics);
        app.at("/api/health").get(api::health);
        app.at("/api/current").get(api::current);
        app.at("/api/:market/at/:date").get(api::history);
        app.listen(&args.addr).await?;
    }
    Ok(())
}
