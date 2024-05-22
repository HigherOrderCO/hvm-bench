use std::collections::BTreeMap;

/// Statistics for various programs, meant to represent the overall benchmarking
/// results for a single revision.
pub struct Stats {
  pub programs: BTreeMap<String, Program>,
}

/// Runtime statistics for a single hvm program, across all interpreted and compiled
/// runtimes.
pub struct Program {
  pub interpreted_c: f64,
  pub interpreted_cuda: f64,
  pub interpreted_rust: f64,
  pub compiled_c: f64,
  pub compiled_cuda: f64,
}
