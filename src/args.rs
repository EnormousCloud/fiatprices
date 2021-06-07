use structopt::StructOpt;
use crate::{Markets, Currencies};

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "fiatprices", about = "API to serve fiat prics of cryptocurrencies. Caches Coingecko so far")]
pub struct Args {
    /// whether to index missing history
    #[structopt(short, long, default_value="1")]
    pub index: u32,
    /// whether to start HTTP API server
    #[structopt(short, long, default_value="1")]
    pub server: u32,
    #[structopt(long, default_value="ethereum,bitcoin", env="MARKETS")]
    pub markets: Markets,
    #[structopt(long, default_value="eur,usd,rub,cny,cad,jpy,gbp", env="CURRENCIES")]
    pub currencies: Currencies,
    #[structopt(short, long, default_value="postgres://postgres:password@localhost/fiatprices", env="DATABASE_URL")]
    pub database_url: String,
    #[structopt(long, default_value="5", env="DATABASE_MAX_CONN")]
    pub database_conn: u32,
    #[structopt(short, long, default_value="0.0.0.0:8080", env="LISTEN")]
    pub addr: String,
}

pub fn parse() -> anyhow::Result<Args> {
    let res =Args::from_args();
    println!("{:?}", res);
    Ok(res)
}
