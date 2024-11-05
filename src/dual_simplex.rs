/// Implementation of two phase minimisation simplex algorithm
/// It starts from the tableau and solves the problem
///
use std::fmt::{self, Display};

use log::debug;

#[derive(Debug, PartialEq)]
enum Phase {
  One,
  Two,
}

// add equality
#[derive(Debug, PartialEq)]
pub struct Matrix {
  phase: Phase,
  variables: usize,
  artificials: usize,
  pub data: Vec<Vec<f64>>,
}

impl Display for Matrix {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let num_rows = self.data.len();
    let num_cols = if num_rows > 0 { self.data[0].len() } else { 0 };
    writeln!(f, "Matrix {}x{}:", num_rows, num_cols)?;

    for row in &self.data {
      for &element in row {
        write!(f, "{:.2}\t", element)?;
      }
      writeln!(f)?;
    }
    Ok(())
  }
}

impl Matrix {
  pub fn new(data: Vec<Vec<f64>>, variables: usize, artificials: usize) -> Self {
    Matrix { phase: Phase::One, data, variables, artificials }
  }

  pub fn get(&self, row: usize, col: usize) -> f64 {
    self.data[row][col]
  }

  pub fn set(&mut self, row: usize, col: usize, val: f64) {
    self.data[row][col] = val;
  }

  pub fn phase_two(&mut self) {
    debug!("Switching to phase two");
    self.phase = Phase::Two;
  }

  fn find_most_positive_in_bottom_row(&self) -> Option<(usize, f64)> {
    let last_row = match self.phase {
      Phase::One => &self.data[self.data.len() - 1],
      Phase::Two => &self.data[self.data.len() - 2],
    };
    debug!("last row full {:?}", last_row);
    let mut found = None;
    let limit = match self.phase {
      Phase::One => 1,
      Phase::Two => self.artificials + 1,
    };
    let last_row = &last_row[..last_row.len() - limit];
    debug!("last row: {:?}", last_row);

    for (col, &x) in last_row.iter().enumerate() {
      if x > 0.0 {
        found = match found {
          Some((_, val)) if x > val => Some((col, x)),
          None => Some((col, x)),
          _ => found,
        };
      }
    }
    found
  }

  fn find_pivot(&self) -> Option<(usize, usize)> {
    let (col, _) = self.find_most_positive_in_bottom_row()?;
    let mut min_ratio = None;
    let mut pivot = None;
    let limit = match self.phase {
      Phase::One => 2,
      Phase::Two => 1,
    };
    let num_rows = self.data.len();
    let num_cols = self.data[0].len();
    for row in 0..num_rows - limit {
      let a = self.get(row, col);
      let b = self.get(row, num_cols - 1);
      // pivot must be positive
      if a > 0.0 && b >= 0.0 {
        let ratio = b / a;
        match min_ratio {
          Some(val) if ratio < val => {
            min_ratio = Some(ratio);
            pivot = Some((row, col));
          }
          None => {
            min_ratio = Some(ratio);
            pivot = Some((row, col));
          }
          _ => (),
        }
      }
    }
    debug!("pivot {:?}", pivot);
    pivot
  }

  fn pivot(&mut self, pivot: (usize, usize)) {
    debug!("Pivoting on {:?}", pivot);
    let (pivot_row, pivot_col) = pivot;
    let pivot_val = self.get(pivot_row, pivot_col);
    let num_rows = match self.phase {
      Phase::One => self.data.len(),
      Phase::Two => self.data.len() - 1,
    };
    let num_cols = self.data[0].len();

    // we need to make the pivot value 1, we divide the row by the pivot value
    for col in 0..num_cols {
      self.set(pivot_row, col, self.get(pivot_row, col) / pivot_val);
    }
    // now we need to make the other values in the column 0
    for row in 0..num_rows {
      if row != pivot_row {
        // our pivot value is 1 so the ratio is the very value we are trying to make 0
        let ratio = self.get(row, pivot_col);
        for col in 0..num_cols {
          self.set(row, col, self.get(row, col) - ratio * self.get(pivot_row, col));
        }
      }
    }
    debug!("{self}");
  }

  pub fn solve(&mut self) -> Result<(), &'static str> {
    // the algorithm is not guaranteed to terminate, we limit the number of iterations
    for _ in 0..1000000 {
      let pivot = self.find_pivot();
      match pivot {
        Some(p) => self.pivot(p),
        None => match self.check_if_we_have_a_solution() {
          true => return Ok(()),
          false => return Err("No feasible solution found"),
        },
      }
    }
    Err("No solution found, iterated too many times")
  }

  pub fn get_solution(&self) -> Vec<f64> {
    let mut solution = vec![0.0; self.variables];
    // the cleared columns get the solution from the last column
    // the other columns get 0
    let num_rows = self.data.len();
    let num_cols = self.data[0].len();
    #[allow(clippy::needless_range_loop)]
    for col in 0..self.variables {
      // the column should contain only one 1, the rest should be 0
      let mut num_zeroes = 0;
      let mut num_ones = 0;
      let mut val = 0.0;
      for row in 0..num_rows {
        if self.get(row, col) == 0.0 {
          num_zeroes += 1;
        } else {
          num_ones += 1;
          val = self.get(row, num_cols - 1);
        }
      }
      if num_zeroes == num_rows - 1 && num_ones == 1 {
        solution[col] = val;
      } else {
        solution[col] = 0.0;
      }
    }

    solution
  }

  pub fn check_if_we_have_a_solution(&self) -> bool {
    match self.phase {
      Phase::One => {
        if let Some(last_row) = self.data.last() {
          if let Some(&last) = last_row.last() {
            let tolerance = 0.0001;
            last.abs() < tolerance
          } else {
            false
          }
        } else {
          false
        }
      }
      Phase::Two => true,
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::tests::init;
  use log::info;

  #[test]
  fn test_artificial_variables_stage_1() {
    init();

    let mut m = Matrix::new(
      vec![
        vec![1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0],
        vec![2.0, -1.0, 0.0, -1.0, 0.0, 0.0, 1.0, 1.0],
        vec![0.0, 3.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0],
        vec![6.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![3.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 2.0],
      ],
      2,
      2,
    );

    info!("{m}");
    assert!(m.solve().is_ok());
    info!("{m}");
  }

  #[test]
  fn test_four_intervals() {
    init();
    // Optimisation for intervals
    // 1. consumption 0, price 1
    // 2. consumption 3, price 2
    // 3. consumption 1, price 2
    // 4. consumption 3, price 1
    // battery starts empty, can not charge more than 1.5
    // battery capacity is 2
    let mut m = Matrix::new(
      vec![
        //   x1   x2   s1   s2   s3   s4   s5   s6   a1   a2   limit
        // max charge constraints
        // x1 <= 1.5
        vec![1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.5],
        // x2 <= 1
        vec![0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        // can not charge more than the battery capacity
        // x1 <= 2
        vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0],
        // x1 + x2 <= 3
        vec![1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 3.0],
        // the overload constraints
        // x1 >= 1
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0],
        // x1 + x2 >= 2
        vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 2.0],
        // the objective function (price)
        vec![-1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        // the intermediate objective function
        vec![2.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 3.0],
      ],
      2,
      2,
    );

    info!("{m}");
    assert!(m.solve().is_ok());
    info!("{m}");
    m.phase_two();
    assert!(m.solve().is_ok());
    assert_eq!(vec![1.5, 0.5], m.get_solution()[0..2]);
  }

  // Tableau for the following minimization problem:
  // maximize p = x + 2y subject to the constraints
  // x <= 1.5
  // y <= 1
  //
  // x >= 1
  // x + y >= 2
  fn tableau_without_max_capacity() -> Matrix {
    init();

    Matrix::new(
      vec![
        //   x1   x2   s1   s2   s3   s4   a1   a2   limit
        // constraints on max load
        vec![1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.5],
        vec![0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        // constraints with artificial variables
        vec![1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0],
        vec![1.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 2.0],
        // objective function
        vec![-1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        // Intermediate objective function
        vec![2.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 3.0],
      ],
      2,
      2,
    )
  }

  #[test]
  fn test_without_max_capacity_individual_steps() {
    init();

    let mut m = tableau_without_max_capacity();
    let pivot = m.find_pivot();
    assert_eq!(pivot, Some((2, 0)));
    m.pivot(pivot.unwrap());

    let pivot = m.find_pivot();
    assert_eq!(pivot, Some((1, 1)));
    m.pivot(pivot.unwrap());

    let pivot = m.find_pivot();
    assert_eq!(pivot, Some((3, 4)));
    m.pivot(pivot.unwrap());

    // check intermediate objective function is zero
    m.phase_two();
    let pivot = m.find_pivot();
    assert_eq!(pivot, Some((0, 3)));
    m.pivot(pivot.unwrap());

    let solution = m.get_solution();
    debug!("solution: {:?}", solution);
    assert_eq!(vec![1.5, 0.5], solution);
  }

  #[test]
  fn test_without_max_capacity() {
    init();

    let mut m = tableau_without_max_capacity();
    info!("{m}");
    assert!(m.solve().is_ok());
    info!("{m}");

    info!("Go to phase 2");
    m.phase_two();
    assert!(m.solve().is_ok());
    info!("{m}");
    let solution = m.get_solution();
    assert_eq!(vec![1.5, 0.5], solution);
  }

  #[test]
  fn test_reverse_coefficients() {
    init();

    let mut m = Matrix::new(
      vec![
        vec![1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.5],
        vec![0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        vec![-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5],
        vec![1.0, 1.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.5],
        vec![-1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![1.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.5],
      ],
      2,
      1,
    );

    info!("{m}");
    assert!(m.solve().is_ok());
    m.phase_two();
    assert!(m.solve().is_ok());
    info!("{m}");
    let solution = m.get_solution();
    assert_eq!(vec![0.5, 0.0], solution);
  }
}
