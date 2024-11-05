
build:
  cargo build

release:
  cargo build --release

#run:
#  cargo run

#debug:
#  RUST_LOG=debug cargo run

test:
  reset; RUST_LOG=debug cargo test dual_simplex::tests::test_without_max_capacity_individual_steps -- --show-output --test-threads=1
# reset; RUST_LOG=debug cargo test calculation::tests::impossible_conditions -- --show-output --test-threads=1
# reset; RUST_LOG=debug cargo test calculation::tests::test_four_intervals -- --show-output --test-threads=1
# reset; cargo test dual_simplex::tests::test_artificial_variables_stage_1 -- --show-output --test-threads=1
# reset; cargo test tableau_creation::tests:: -- --show-output --test-threads=1

testall:
  reset; RUST_LOG=debug cargo test -- --show-output --test-threads=1

fmt:
  cargo fmt

calc:
  reset; cargo test calculation::tests:: -- show-output --test-threads=1 --nocapture

