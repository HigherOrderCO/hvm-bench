use std::{io::Write, path::Path, process::Command, time::Duration};

use anyhow::{Context, Result};
use tempfile::{NamedTempFile, TempDir};

use crate::{
  ext::{CommandExt, NamedTempFileExt},
  stats::Timing,
};

const TIME_PREFIX: &str = "- TIME: ";

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
fn interpreted<P, Q>(hvm_bin: P, mode: &str, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let Some(stdout) = Command::new(hvm_bin.as_ref())
    .arg(mode)
    .arg(program.as_ref())
    .status_stdout_timeout(timeout)?
  else {
    return Ok("timeout".to_string());
  };

  parse_stdout(&stdout).context("parse")
}

pub fn interpreted_c<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  interpreted(hvm_bin, "run-c", program, timeout)
}

pub fn interpreted_cuda<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  interpreted(hvm_bin, "run-cu", program, timeout)
}

pub fn interpreted_rust<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  interpreted(hvm_bin, "run", program, timeout)
}

/// Generates a file to be compiled.
fn generate_program<P, Q>(hvm_bin: P, mode: &str, program: Q) -> Result<String>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let output = Command::new(hvm_bin.as_ref())
    .arg(mode)
    .arg(program.as_ref())
    .status_stdout();

  output
}

fn compile_and_run(compiler: &str, file: &Path, args: &[&str], timeout: Duration) -> Result<Timing> {
  let bin_dir = TempDir::with_prefix("hvm-bench-compile-").context("tempdir")?;
  let binary = bin_dir.path().join("bin");

  Command::new(compiler)
    .arg(file)
    .args(args)
    .arg("-o")
    .arg(&binary)
    .check_success()
    .context("compile")?;

  let Some(stdout) = Command::new(binary).status_stdout_timeout(timeout)? else {
    return Ok("timeout".to_string());
  };

  parse_stdout(&stdout).context("parse")
}

pub fn compiled_c<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let mut c_file = NamedTempFile::with_suffix(".c")?;
  let c_code = generate_program(hvm_bin, "gen-c", program).context("generate program")?;
  c_file.write_all(c_code.as_bytes()).context("write")?;

  compile_and_run("gcc", c_file.path(), &["-lm", "-O2"], timeout).context("compile and run")
}

pub fn compiled_cuda<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let mut cu_file = NamedTempFile::with_suffix(".cu")?;
  let cu_code = generate_program(hvm_bin, "gen-cu", program).context("generate program")?;
  cu_file.write_all(cu_code.as_bytes()).context("write")?;

  compile_and_run("nvcc", cu_file.path(), &["-w", "-O3"], timeout).context("compile and run")
}
