use std::{io::Write, path::Path, process::Command};

use anyhow::{Context, Result};
use tempfile::{Builder, TempDir};

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

  let stdout = run_program(hvm_bin, mode, program).context("run")?;

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

/// Generates a file to be compiled.
fn generate_program<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, mode: &str, program: Q) -> Result<String> {
  let hvm_bin = hvm_bin.as_ref();
  let program = program.as_ref();

  run_program(hvm_bin, mode, program).with_context(|| format!("{hvm_bin:?} {mode} {program:?}"))
}

fn compile_and_run(compiler: &str, file: &Path, args: &[&str]) -> Result<Timing> {
  let bin_dir = TempDir::with_prefix("hvm-bench-compile-").context("tempdir")?;
  let binary = bin_dir.path().join("bin");

  let status = Command::new(compiler)
    .arg(file)
    .args(args)
    .arg("-o")
    .arg(&binary)
    .status()
    .context("compile")?;
  if !status.success() {
    anyhow::bail!("compiler exited with non-zero status {}", status);
  }

  let output = Command::new(&binary).output().context("run")?;
  if !output.status.success() {
    anyhow::bail!("runner exited with non-zero status {}", output.status);
  }

  parse_stdout(&String::from_utf8_lossy(&output.stdout).into_owned()).context("parse")
}

pub fn compiled_c<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, program: Q) -> Result<Timing> {
  let mut c_file = Builder::new().suffix(".c").tempfile().context("tempfile")?;
  let c_code = generate_program(hvm_bin, "gen-c", program).context("generate program")?;
  c_file.write_all(c_code.as_bytes()).context("write")?;

  compile_and_run("gcc", &c_file.path(), &["-lm"]).context("compile and run")
}

pub fn compiled_cuda<P: AsRef<Path>, Q: AsRef<Path>>(hvm_bin: P, program: Q) -> Result<Timing> {
  let mut cu_file = Builder::new().suffix(".cu").tempfile().context("tempfile")?;
  let cu_code = generate_program(hvm_bin, "gen-cu", program).context("generate program")?;
  cu_file.write_all(cu_code.as_bytes()).context("write")?;

  compile_and_run("nvcc", &cu_file.path(), &[]).context("compile and run")
}
