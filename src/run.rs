use std::{
  io::{Read, Write},
  path::Path,
  process::{Command, Stdio},
  time::Duration,
};

use anyhow::{Context, Result};
use tempfile::{Builder, TempDir};
use wait_timeout::ChildExt;

use crate::stats::Timing;

const TIME_PREFIX: &str = "- TIME: ";

/// Runs `hvm_bin` on a program with a given `mode` and returns its stdout. The
/// `mode` is provided as a positional argument to `hvm_bin`, so it is expected
/// to be one of the `hvm` commands (`run`, `run-c`, `run-cu`, etc).
///
/// # Errors
///
/// This will return an error if:
/// - the exit status is non-zero.
fn run_program(hvm_bin: &Path, mode: &str, program: &Path, timeout: Duration) -> Result<Option<String>> {
  let mut child = Command::new(hvm_bin)
    .arg(mode)
    .arg(program)
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .context("spawn")?;

  let mut stdout = child.stdout.take().context("stdout")?;

  let status = child.wait_timeout(timeout).context("wait")?;
  let Some(status) = status else {
    return Ok(None);
  };
  if !status.success() {
    anyhow::bail!("non-zero exit status {}", status);
  }

  let mut output = String::new();
  stdout.read_to_string(&mut output).context("read")?;

  Ok(Some(output))
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
fn interpreted<P, Q>(hvm_bin: P, mode: &str, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let hvm_bin = hvm_bin.as_ref();
  let program = program.as_ref();

  let Some(stdout) = run_program(hvm_bin, mode, program, timeout).context("run")? else {
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
  let hvm_bin = hvm_bin.as_ref();
  let program = program.as_ref();

  run_program(hvm_bin, mode, program, Duration::from_secs(600))
    .with_context(|| format!("{hvm_bin:?} {mode} {program:?}"))?
    .context("timeout")
}

fn compile_and_run(compiler: &str, file: &Path, args: &[&str], timeout: Duration) -> Result<Timing> {
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

  let mut child = Command::new(binary)
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .context("spawn")?;

  let mut stdout = child.stdout.take().context("stdout")?;

  let status = child.wait_timeout(timeout).context("wait")?;
  let Some(status) = status else {
    return Ok("timeout".to_string());
  };
  if !status.success() {
    anyhow::bail!("non-zero exit status {}", status);
  }

  let mut output = String::new();
  stdout.read_to_string(&mut output).context("read")?;

  parse_stdout(&output).context("parse")
}

pub fn compiled_c<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let mut c_file = Builder::new().suffix(".c").tempfile().context("tempfile")?;
  let c_code = generate_program(hvm_bin, "gen-c", program).context("generate program")?;
  c_file.write_all(c_code.as_bytes()).context("write")?;

  compile_and_run("gcc", c_file.path(), &["-lm", "-O2"], timeout).context("compile and run")
}

pub fn compiled_cuda<P, Q>(hvm_bin: P, program: Q, timeout: Duration) -> Result<Timing>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let mut cu_file = Builder::new().suffix(".cu").tempfile().context("tempfile")?;
  let cu_code = generate_program(hvm_bin, "gen-cu", program, timeout).context("generate program")?;
  cu_file.write_all(cu_code.as_bytes()).context("write")?;

  compile_and_run("nvcc", cu_file.path(), &["-w", "-O3"], timeout).context("compile and run")
}
