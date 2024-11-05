use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::Args;

#[derive(Debug, Deserialize)]
struct Consumption {
  start: DateTime<Utc>,
  end: DateTime<Utc>,
  #[serde(rename = "consumption_average_power_interval")]
  power: f64,
}

#[derive(Debug, Deserialize)]
struct Price {
  start: DateTime<Utc>,
  end: DateTime<Utc>,
  #[serde(rename = "market_price_per_kwh")]
  value: f64,
}

#[derive(Debug, Deserialize)]
struct Forecasts {
  forecasts: Vec<Consumption>,
}

#[derive(Debug, Deserialize)]
struct Prices {
  prices: Vec<Price>,
}

pub struct Data {
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
  pub power: f64,
  pub price: f64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub max_consumption: f64,
  pub battery_capacity: f64,
  pub battery_max_charge: f64,
  pub battery_initial_charge: f64,
  pub battery_efficiency: f64,
  pub battery_final_charge: f64,
}

enum FileType {
  Json,
  Toml,
}
fn read_file_and_parse<T>(filename: &str, file_type: FileType) -> T
where
  T: serde::de::DeserializeOwned,
{
  let text = match std::fs::read_to_string(filename) {
    Ok(json) => json,
    Err(e) => {
      eprintln!("Unable to read file: {}, {}", filename, e);
      std::process::exit(1);
    }
  };
  match file_type {
    FileType::Json => match serde_json::from_str(&text) {
      Ok(f) => f,
      Err(e) => {
        eprintln!("Unable to parse Json from file {}: {}", filename, e);
        std::process::exit(1);
      }
    },
    FileType::Toml => match toml::from_str(&text) {
      Ok(f) => f,
      Err(e) => {
        eprintln!("Unable to parse Toml from file {}: {}", filename, e);
        std::process::exit(1);
      }
    },
  }
}

// read the required data from the files and perform some basic checks
pub fn read_data(args: Args) -> (Vec<Data>, Config) {
  let forecast: Forecasts = read_file_and_parse(&args.consumption, FileType::Json);
  let price: Prices = read_file_and_parse(&args.prices, FileType::Json);
  debug!("Read {}, {} records", forecast.forecasts.len(), price.prices.len());

  let forecasts = forecast.forecasts;
  let prices = price.prices;

  if forecasts.is_empty() {
    panic!("No consumption data");
  }
  if prices.is_empty() {
    panic!("No price data");
  }

  // check that the start and the end of the time series is the same for both
  if forecasts.first().unwrap().start != prices.first().unwrap().start {
    panic!("Start of time series is not the same for both forecasts and prices");
  }
  if forecasts.last().unwrap().end != prices.last().unwrap().end {
    panic!("End of time series is not the same for both forecasts and prices");
  }
  debug!(
    "Time series starts at {} and ends at {}, consumption and price time series overlap",
    forecasts[0].start,
    forecasts.last().unwrap().end
  );

  let mut joined_data: Vec<Data> = Vec::with_capacity(forecasts.len());
  // join the power intervals with the prices. There is one price for four power intervals
  for (i, val) in forecasts.iter().enumerate() {
    joined_data.push(Data { start: val.start, end: val.end, power: val.power, price: prices[i / 4].value });
  }

  // read the conditions data
  let config: Config = read_file_and_parse(&args.config, FileType::Toml);

  (joined_data, config)
}

/// Output data is a JSON file with energy in and from the battery
#[derive(Debug, Serialize)]
pub struct Plan {
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
  pub energy_from_battery_wh: f64,
  pub energy_to_battery_wh: f64,
}

#[derive(Debug, Serialize)]
pub struct Out {
  pub planning: Vec<Plan>,
}

pub fn print_output(planning: Vec<Plan>) {
  let out = Out { planning };
  let json = serde_json::to_string_pretty(&out).expect("Unable to serialize output");
  println!("{}", json);
}
