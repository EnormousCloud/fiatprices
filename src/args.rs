#[derive(Debug, Clone)]
pub struct Args {
    pub markets: Vec<String>,
    pub currencies: Vec<String>,
    pub database_url: String,
    pub database_conn: u32,
}

const USAGE: &str = "\
USAGE:
    fiatprices [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --db-url     DB_URL   Postgres database URL
  --db-maxconn NUMBER   max number of database connections
";

fn vec_str(input: Vec<&str>) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for &s in input.iter() {
        out.push(String::from(s))
    }
    out
}

pub fn parse() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", USAGE);
        std::process::exit(0);
    }

    let res = Args {
        markets: vec_str(vec!["bitcoin", "ethereum"]),
        currencies: vec_str(vec!["eur", "usd", "rub", "cny", "cad", "jpy", "gbp"]),
        database_url: pargs
            .value_from_str("--db-url")
            .unwrap_or("postgres://postgres:password@localhost/fiatprices".to_owned()),
        database_conn: pargs.value_from_str("--db-maxconn").unwrap_or(5),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }
    Ok(res)
}
