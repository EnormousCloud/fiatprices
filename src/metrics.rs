use prometheus::{Encoder, Gauge, Opts, Registry, TextEncoder};
use std::collections::HashMap;

pub type CurrentMarkets = HashMap<String, HashMap<String, f64>>;

pub fn output(src: CurrentMarkets) -> String {
    let encoder = TextEncoder::new();
    let labels = HashMap::new();
    let sr = Registry::new_custom(Some("fiatprices".to_string()), Some(labels)).unwrap();
    
    for (market_name, market) in src {
        for (currency, value) in market {
            let gauge_opts = Opts::new("price", "price")
                .const_label("market", market_name.as_str())
                .const_label("currency", currency.as_str());
            let gauge = Gauge::with_opts(gauge_opts).unwrap();
            gauge.set(value);
            sr.register(Box::new(gauge.clone())).unwrap();
        }
    }

    let mut buffer = Vec::<u8>::new();
    encoder.encode(&sr.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer.clone()).unwrap()
}