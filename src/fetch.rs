use anyhow::Result;

pub fn fetch_current(markets: &Vec<String>, currencies: &Vec<String>) -> Result<()> {
    // "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd%2Ceur%2Crub%2Ccny%2Ccad%2Cjpy%2Cgbp%2Caud"
    Ok(())
}

pub fn fetch_history(market: &str, currencies: &Vec<String>) -> Result<()> {
    // https://api.coingecko.com/api/v3/coins/bitcoin,ethereum/history?date=01-01-2017
    Ok(())
}
