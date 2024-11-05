use clap::Parser;
use data::print_output;

mod calculation;
mod data;
mod dual_simplex;
mod tableau_creation;

#[derive(Parser)]
struct Args {
  #[arg(short, long, default_value = "consumption.json", help = "json file with the predicted power consumption")]
  consumption: String,
  #[arg(short, long, default_value = "prices.json", help = "json file with the predicted prices")]
  prices: String,
  #[arg(
    short = 'i',
    long,
    default_value = "config.toml",
    help = "toml file with customer configuration, max power, battery capacity, etc."
  )]
  config: String,
}

fn main() {
  let args = Args::parse();
  let (data, config) = data::read_data(args);
  let planning = calculation::calculation(&data, &config).expect("Calculation failed");
  print_output(planning);
}

#[cfg(test)]
mod tests {
  use std::sync::Once;

  static INIT: Once = Once::new();

  pub fn init() {
    INIT.call_once(|| {
      let _ = env_logger::builder().is_test(true).format_timestamp(None).try_init();
    });
  }
}
