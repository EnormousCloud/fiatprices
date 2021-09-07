pub mod api;
pub mod args;
pub mod db;
pub mod exporter;
pub mod fetch;

use chrono::{Datelike, NaiveDate, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Market {
    pub name: String,
    pub earliest: NaiveDate<Utc>,
}

impl Market {
    pub fn new(src: &str) -> Self {
        let parts = src.split(":");
        let now = Utc::now();
        let start = Utc.ymd(now.year(), 1, 1).and_hms(0, 0, 0);
        let (name, earliest) = if parts.len() == 0 {
            (src.to_owned(), start)
        } else {
            let dt = match NaiveDate::parse_from_str(parts[1], "%Y-%m-%d") {
                Ok(x) => x,
                Err(e) => start,
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
pub struct Markets(Vec<String>);
impl std::str::FromStr for Markets {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Markets(s.split(",").map(|x| x.trim().to_owned()).collect()))
    }
}
impl Markets {
    pub fn as_vec(&self) -> Vec<String> {
        self.0.clone()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, std::string::String> {
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

    exporter::init(&mut conn, &args.markets, &args.currencies).await?;
    if args.index > 0 {
        exporter::update_history(&mut conn, &args.markets, &args.currencies).await?;
    }
    if args.server > 0 {
        let state = State {
            db_pool: pool,
            markets: args.markets.clone(),
            currencies: args.currencies.clone(),
        };
        let mut app = tide::with_state(state);
        app.with(LogMiddleware {});
        // app.with(tide_tracing::TraceMiddleware::new());
        app.at("/api/health").get(api::health);
        app.at("/api/current").get(api::current);
        app.at("/api/:market/at/:date").get(api::history);
        app.listen(args.addr.as_str()).await?;
    }
    Ok(())
}
