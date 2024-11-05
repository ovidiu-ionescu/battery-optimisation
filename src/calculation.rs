use log::debug;

use crate::{
  data::{Config, Data, Plan},
  dual_simplex::Matrix,
  tableau_creation::build_tableau,
};

pub fn calculation(data: &[Data], config: &Config) -> Result<Vec<Plan>, String> {
  let (tableau, variables, artificials) = build_tableau(data, config);
  let mut matrix = Matrix::new(tableau, variables, artificials);

  matrix.solve()?;
  matrix.phase_two();
  matrix.solve()?;
  let solution = matrix.get_solution();
  let count_vars = data.iter().filter(|d| d.power <= config.max_consumption).count();
  debug!("The solution is: {:?}", &solution[0..count_vars]);
  // make the plan
  let mut planning: Vec<Plan> = Vec::with_capacity(data.len());
  // if we use more than the limit we get it from battery, otherwise we charge the battery
  let mut solution_offset = 0;
  for d in data {
    if d.power <= config.max_consumption {
      planning.push(Plan {
        start: d.start,
        end: d.end,
        energy_to_battery_wh: solution[solution_offset] / 4.0,
        energy_from_battery_wh: 0.0,
      });
      solution_offset += 1;
    } else {
      planning.push(Plan {
        start: d.start,
        end: d.end,
        energy_to_battery_wh: 0.0,
        energy_from_battery_wh: (d.power - config.max_consumption) / 4.0,
      });
    }
  }
  Ok(planning)
}

#[cfg(test)]
mod tests {
  use crate::tests::init;

  use super::*;
  use chrono::Utc;
  use log::info;

  #[test]
  fn test_four_intervals() {
    init();

    let start = Utc::now();
    let end = Utc::now();
    let data = vec![
      Data { start, end, power: 0.0, price: 1.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 1.0, price: 2.0 },
      Data { start, end, power: 3.0, price: 0.9 },
    ];
    let config = Config {
      max_consumption: 2.0,
      battery_capacity: 2.0 / 4.0,
      battery_max_charge: 1.5,
      battery_initial_charge: 1.5 / 4.0,
      battery_efficiency: 0.9,
      battery_final_charge: 0.0,
    };
    let (tableau, v, a) = build_tableau(&data, &config);
    let mut matrix = Matrix::new(tableau, v, a);
    info!("This is the initial matrix");
    info!("{matrix}");
    assert!(matrix.solve().is_ok());
    info!("going to phase 2");
    matrix.phase_two();
    assert!(matrix.solve().is_ok());
    let solution = matrix.get_solution();
    let expected = vec![0.5555555, 0.0];
    let tolerance = 0.0001;
    for (i, s) in solution[0..2].iter().enumerate() {
      assert!((s - expected[i]).abs() < tolerance);
    }
  }

  #[test]
  fn test_five_intervals_and_battery_recharge() {
    init();

    let start = Utc::now();
    let end = Utc::now();
    let data = vec![
      Data { start, end, power: 0.0, price: 1.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 1.0, price: 2.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 0.0, price: 1.0 },
    ];
    let config = Config {
      max_consumption: 2.0,
      battery_capacity: 2.0 / 4.0,
      battery_max_charge: 1.5,
      battery_initial_charge: 1.5 / 4.0,
      battery_efficiency: 0.9,
      battery_final_charge: 0.5 / 4.0,
    };
    let (tableau, v, a) = build_tableau(&data, &config);
    let mut matrix = Matrix::new(tableau, v, a);
    info!("This is the initial matrix");
    info!("{matrix}");
    assert!(matrix.solve().is_ok());
    info!("going to phase 2");
    matrix.phase_two();
    assert!(matrix.solve().is_ok());
    let solution = matrix.get_solution();
    info!("The solution is: {:?}", &solution);
    let expected = vec![0.5555555, 0.0, 0.5555555];
    let tolerance = 0.0001;
    for (i, s) in solution[0..expected.len()].iter().enumerate() {
      assert!((s - expected[i]).abs() < tolerance);
    }
  }

  #[test]
  fn impossible_conditions() {
    init();

    let start = Utc::now();
    let end = Utc::now();
    let data = vec![
      Data { start, end, power: 0.0, price: 1.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 1.0, price: 2.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 0.0, price: 1.0 },
    ];
    let config = Config {
      max_consumption: 2.0,
      battery_capacity: 2.0 / 4.0,
      battery_max_charge: 1.5,
      battery_initial_charge: 1.5 / 4.0,
      battery_efficiency: 0.9,
      // too high to be possible
      battery_final_charge: 100.0,
    };
    let (tableau, v, a) = build_tableau(&data, &config);
    let mut matrix = Matrix::new(tableau, v, a);
    info!("This is the initial matrix");
    info!("{matrix}");
    assert!(matrix.solve().is_err());
  }
}
