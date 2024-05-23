use std::{path::Path, process::Command};

use anyhow::{Context, Result};

use crate::stats::Timing;

const TIME_PREFIX: &str = "- TIME: ";

/// Runs `hvm_bin` on a program with a given `mode` and returns its stdout. The
/// `mode` is provided as a positional argument to `hvm_bin`, so it is expected
/// to be one of the `hvm` commands (`run`, `run-c`, `run-cu`, etc).
///
/// TODO: this function should be blocking, but should timeout after N seconds.
///
/// # Errors
///
/// This will return an error if:
/// - the exit status is non-zero.
fn run_program(hvm_bin: &Path, mode: &str, program: &Path) -> Result<String> {
  let output = Command::new(hvm_bin)
    .arg(mode)
    .arg(program)
    .output()
    .context("output")?;

  if !output.status.success() {
    anyhow::bail!("non-zero exit status {}", output.status);
  }

  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Returns the timing line of an `hvm` run.
fn parse_stdout(stdout: &str) -> Result<Timing> {
  for line in stdout.lines() {
    if let Some(timing) = line.strip_prefix(TIME_PREFIX) {
      return Ok(timing.to_string());
    }
  }

  anyhow::bail!("no line with {TIME_PREFIX:?} found")
}

/// Executes `hvm_bin mode program`, parsing hvm's timing output. an interpreted
/// mode, without an additional C compilation step.
fn interpreted<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, mode: &str, program: Q) -> Result<Timing> {
  let hvm_bin = hvm_bin.as_ref();
  let program = program.as_ref();

  let stdout = run_program(hvm_bin, mode, program).with_context(|| format!("{hvm_bin:?} {mode} {program:?}"))?;

  parse_stdout(&stdout).context("parse")
}

pub fn interpreted_c<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, program: Q) -> Result<Timing> {
  interpreted(hvm_bin, "run-c", program)
}

pub fn interpreted_cuda<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, program: Q) -> Result<Timing> {
  interpreted(hvm_bin, "run-cu", program)
}

pub fn interpreted_rust<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, program: Q) -> Result<Timing> {
  interpreted(hvm_bin, "run", program)
}
