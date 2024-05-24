use std::{
  io::{Read, Write},
  process::{Child, Command, ExitStatus, Stdio},
  time::Duration,
};

use anyhow::{Context, Result};
use tempfile::{Builder, NamedTempFile};
use wait_timeout::ChildExt as WaitExt;

#[extend::ext]
pub impl ExitStatus {
  fn check_success(&self) -> Result<()> {
    if !self.success() {
      anyhow::bail!("exited with non-zero status {self}");
    }

    Ok(())
  }
}

#[extend::ext]
pub impl Child {
  /// Returns an error if the exit status was non-zero.
  fn check_success(&mut self) -> Result<()> {
    self.wait().context("wait")?.check_success()
  }

  /// Returns an error if the exit status was non-zero. On timeout, returns
  /// `Ok(None)`.
  fn check_success_timeout(&mut self, timeout: Duration) -> Result<Option<()>> {
    let Some(status) = self.wait_timeout(timeout).context("wait")? else {
      self.kill().expect("failed to kill child after timeout");

      return Ok(None);
    };

    status.check_success()?;

    Ok(Some(()))
  }
}

#[extend::ext]
pub impl Command {
  fn check_success(&mut self) -> Result<()> {
    self.status().context("status")?.check_success()
  }

  /// Runs the command, capturing only stdout, returning an error on non-zero
  /// exit.
  fn status_stdout(&mut self) -> Result<String> {
    // NOTE(enricozb): for some reason, writing this using a child that's spawned
    // and waited on does not work for the compilers `gcc` and `nvcc`, they just
    // hang on `wait()`.
    let output = self.output().context("output")?;
    output.status.check_success()?;

    std::io::stderr().write_all(&output.stderr).context("write")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  }

  /// Runs the command, capturing only stdout, returning an error on non-zero
  /// exit, or `Ok(None)` on timeout.
  fn status_stdout_timeout(&mut self, timeout: Duration) -> Result<Option<String>> {
    let mut child = self.stdout(Stdio::piped()).spawn().context("spawn")?;
    let mut stdout = child.stdout.take().context("stdout")?;

    if child.check_success_timeout(timeout)?.is_none() {
      return Ok(None);
    }

    let mut output = String::new();
    stdout.read_to_string(&mut output).context("read")?;

    Ok(Some(output))
  }
}

#[extend::ext]
pub impl NamedTempFile {
  fn with_suffix(suffix: &str) -> Result<NamedTempFile> {
    Builder::new().suffix(suffix).tempfile().context("tempfile")
  }
}
