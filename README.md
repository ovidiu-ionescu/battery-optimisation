# Optimise electricity consumption using a battery

## Introduction
The program will create a plan for charging and discharging a battery
in order to keep the electricity consumption under a certain limit.  
It will try to keep the price of charging the battery as low as possible.

Further optimisations can be included to use the battery even when the
electricity consumption is below the limit but using the battery is cheaper.  
That part is not implemented yet.

Because all the inequalities are linear, the problem can be solved using 
linear programming. The assignment prohibits using an off the shelf
library for that so I had to implement the simplex algorithm myself.

## How to run
The program will try to read the data from three files:
- consumption.json - the predicted electricity consumption for each 15 minutes
- prices.json - the predicted electricity prices for each hour
- config.toml - max power use limit and battery caracteristics, battery initial and final charge

The program uses clap to parse the command line arguments. This way you can override the default
file names for all three files.
```bash
cargo run -- -c consumption.json -p prices.json -i config.toml
```

## Create tableau from the data
The data is read from the files and the tableau is created by the code in the module
tableau_creation.rs. The tableau is a matrix that will be used by the simplex algorithm.

## Solve the tableau
The module dual_simplex.rs contains the implementation of the dual simplex algorithm.
It optimises in two steps: first it finds a feasible solution and then it optimises it.
The reason it needs two steps is that then inequalities for the discharge are different
and the algorithm needs to find a feasible solution first. If the battery is too small
then it might not be possible to compensate.

## Calculate the plan
Once the tableau is solved, the plan is calculated in the module calculation.rs. 

