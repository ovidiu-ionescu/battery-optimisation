use log::debug;

use crate::data::{Config, Data};

/// Creates the tableau for the dual simplex minimization algorithm
/// The tableau is a matrix with the following structure:
/// 1. loading constraints for max battery charge and max power
/// 2. loading constraints for the battery capacity
/// 3. constraints for the battery discharge, needs to compensate for the overload
/// 4. loading constraints for the final battery value
/// 5. price optimization
/// 6. intermediate goal (required because 5. has artificial variables)
///
pub fn build_tableau(data: &[Data], config: &Config) -> (Vec<Vec<f64>>, usize, usize) {
  // the battery capacity is per hour so it will become per quarter by multiplying by 4
  let b0 = config.battery_initial_charge * 4.0; // instead of MWh we have MW15minutes
  let b_max = config.battery_capacity * 4.0;
  let b_final = config.battery_final_charge * 4.0;
  debug!("b0: {b0}");

  let count_vars = data.iter().filter(|d| d.power <= config.max_consumption).count();
  let count_over = data.len() - count_vars;
  // we have two criteria, optimisation and feasibility
  let rows = 2 * count_vars + count_over + 1 + 2;
  // we get an s per equation. For each underload interval 2 equations (max power and max battery)
  // for each overload 1 equation (need enough juice in the battery)
  // one equation for final value of the battery
  let num_s = 2 * count_vars + count_over + 1;
  let num_max_a = count_over + 1;
  let cols = count_vars + num_s + num_max_a + 1;
  debug!("rows: {}, cols: {}", cols, rows);
  let negate = |v: &mut [f64]| {
    for z in v.iter_mut() {
      if *z != 0.0 {
        *z = -*z;
      }
    }
  };
  let mut result: Vec<Vec<f64>> = Vec::with_capacity(rows);
  // equations for limiting the charge
  let mut line_count = 0;
  let mut x_vs_interval_offset = 0;
  let mut a_offset = count_vars + num_s;
  // equation for max power charge.
  for (i, d) in data.iter().enumerate() {
    if d.power >= config.max_consumption {
      x_vs_interval_offset += 1;
      continue;
    }
    let mut equation: Vec<f64> = vec![0.0; cols];
    // the x
    equation[i - x_vs_interval_offset] = 1.0;
    // the s
    equation[count_vars + line_count] = 1.0;
    line_count += 1;
    // the limit
    equation[cols - 1] = config.battery_max_charge.min(config.max_consumption - d.power);
    result.push(equation);
  }
  let mut intermediate: Vec<f64> = vec![0.0; cols];
  // equations for the limit of the battery capacity
  let mut x_vs_interval_offset = 0;
  let mut discharge = 0.0;
  for (i, d) in data.iter().enumerate() {
    if d.power >= config.max_consumption {
      x_vs_interval_offset += 1;
      discharge += d.power - config.max_consumption;
      continue;
    }
    let mut equation: Vec<f64> = vec![0.0; cols];
    // the x
    #[allow(clippy::needless_range_loop)]
    for col in 0..i - x_vs_interval_offset + 1 {
      equation[col] = config.battery_efficiency;
    }
    // the s
    equation[count_vars + line_count] = 1.0;
    // the limit
    let limit = b_max + discharge - b0;
    equation[cols - 1] = limit;
    if limit < 0.0 {
      negate(&mut equation);
      // set the a
      debug!("a_offset: {}", a_offset);
      equation[a_offset] = 1.0;
      a_offset += 1;
    }
    line_count += 1;
    result.push(equation);
  }

  // equations for discharging
  // we'll build the intermediate goal at the same time as it is a running sum
  let mut x_vs_interval_offset = 0;
  let mut discharge = 0.0;
  for (i, d) in data.iter().enumerate() {
    if d.power >= config.max_consumption {
      x_vs_interval_offset += 1;
      discharge += d.power - config.max_consumption;
      let limit = discharge - b0;
      let mut equation: Vec<f64> = vec![0.0; cols];

      // the x
      for j in 0..i - x_vs_interval_offset + 1 {
        equation[j] = config.battery_efficiency;
        if limit >= 0.0 {
          intermediate[j] += equation[j];
        };
      }
      // the s
      equation[count_vars + line_count] = -1.0;
      // the limit
      equation[cols - 1] = limit;
      if limit < 0.0 {
        negate(&mut equation);
      } else {
        // set the a
        equation[a_offset] = 1.0;
        a_offset += 1;
        intermediate[count_vars + line_count] = -1.0;
        intermediate[cols - 1] += limit;
      }
      line_count += 1;
      result.push(equation);
    }
  }

  // equation for the final battery value
  // b0 + sum(efficiency * xi) - sum(overload) >= b_final
  let limit = b_final - b0 + discharge;
  let mut equation: Vec<f64> = vec![0.0; cols];
  for i in 0..count_vars {
    equation[i] = config.battery_efficiency;
    if limit >= 0.0 {
      intermediate[i] += config.battery_efficiency;
    }
  }
  if limit >= 0.0 {
    // the s
    equation[count_vars + line_count] = -1.0;
    intermediate[count_vars + line_count] = -1.0;
    // the a
    equation[a_offset] = 1.0;
    a_offset += 1;
    // the limit
    equation[cols - 1] = limit;
    intermediate[cols - 1] += limit;
  } else {
    negate(&mut equation);
    // the s
    equation[count_vars + line_count] = 1.0;
    // the limit
    equation[cols - 1] = -limit;
  }
  result.push(equation);

  // price, the optimization function
  let mut x_vs_interval_offset = 0;
  let mut equation: Vec<f64> = vec![0.0; cols];
  for (i, d) in data.iter().enumerate() {
    if d.power >= config.max_consumption {
      x_vs_interval_offset += 1;
      continue;
    }
    equation[i - x_vs_interval_offset] = -d.price;
  }
  result.push(equation);
  result.push(intermediate);

  // trim the unused a columns
  for r in result.iter_mut() {
    r[a_offset] = r[cols - 1];
    r.truncate(a_offset + 1);
  }
  (result, count_vars, a_offset - count_vars - num_s)
}

// test module
#[cfg(test)]
mod tests {

  use super::*;
  use crate::tests::init;
  use chrono::Utc;
  use log::info;

  #[test]
  fn test_build_tableau() {
    init();
    let start = Utc::now();
    let end = Utc::now();
    let data = vec![
      Data { start, end, power: 0.0, price: 1.0 },
      Data { start, end, power: 3.0, price: 2.0 },
      Data { start, end, power: 1.0, price: 2.0 },
      Data { start, end, power: 3.0, price: 1.0 },
    ];
    let config = Config {
      max_consumption: 2.0,
      battery_capacity: 2.0 / 4.0,
      battery_max_charge: 1.5,
      battery_initial_charge: 1.5 / 4.0,
      battery_efficiency: 0.9,
      battery_final_charge: 0.0,
    };
    let (result, v, a) = build_tableau(&data, &config);
    for r in result.iter() {
      info!("{:?}", r);
    }
    assert_eq!(v, 2);
    assert_eq!(a, 2);
    assert_eq!(
      result,
      [
        //x1  x2   s1   s2   s3   s4   s5   s6   s7   a1   a2   limit
        [1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.5], // cap on charge x1
        [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // cap on charge x2
        [0.9, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5], // max battery x1
        [0.9, 0.9, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.5], // max battery x2
        // b0 + e*x1 >= o1 -> 1.5 +0.9 *x1 >= 1 -> 0.9 * x1 >= -0.5
        // -> -0.9 * x1 < 0.5 -> -0.9 *x1 + s5 = 0.5
        [-0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5], // enough power o1
        [0.9, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.5], // enough power o2
        [0.9, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.5], // final battery                                                    //
        [-1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // total price
        [1.8, 1.8, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0]  // intermediate
      ]
    );
  }
}
