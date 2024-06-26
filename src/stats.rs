use std::collections::BTreeMap;

use anyhow::Result;

/// The time reported by `hvmc`, unparsed.
pub type Timing = String;

/// Statistics for various programs, meant to represent the overall benchmarking
/// results for a single revision.
#[derive(Default)]
pub struct Stats {
  pub programs: BTreeMap<String, Program>,
}

/// Runtime statistics for a single hvm program, across all interpreted and
/// compiled runtimes.
pub struct Program {
  pub compiled_c: Result<Timing>,
  pub compiled_cuda: Result<Timing>,
  pub interpreted_c: Result<Timing>,
  pub interpreted_cuda: Result<Timing>,
  pub interpreted_rust: Result<Timing>,
}

impl Program {
  pub fn compiled(&self, runtime: &str) -> &Result<Timing> {
    match runtime {
      "c" => &self.compiled_c,
      "cuda" => &self.compiled_cuda,

      _ => panic!("unexpected runtime: {runtime}"),
    }
  }

  pub fn interpreted(&self, runtime: &str) -> &Result<Timing> {
    match runtime {
      "c" => &self.interpreted_c,
      "cuda" => &self.interpreted_cuda,
      "rust" => &self.interpreted_rust,

      _ => panic!("unexpected runtime: {runtime}"),
    }
  }
}
