use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::stats::Stats;

const GIT_URL: &str = "https://github.com/HigherOrderCO/hvm.git";

pub struct Bench {
  /// Local hvm directory.
  local_dir: PathBuf,
  /// Remote revisions.
  remote_revs: Vec<String>,
  /// Statistics collected for each revision.
  stats: BTreeMap<String, Stats>,
  /// Temporary directory for binaries and remote repo.
  tempdir: TempDir,
}

impl Bench {
  pub fn new(local_dir: PathBuf, remote_revs: Vec<String>) -> Result<Self> {
    let tempdir = TempDir::with_prefix("hvm-bench-").context("tempdir")?;

    fs::create_dir(tempdir.path().join("repo")).context("create_dir repo")?;
    fs::create_dir(tempdir.path().join("bin")).context("create_dir bin")?;

    Ok(Self {
      local_dir,
      remote_revs,
      stats: BTreeMap::new(),
      tempdir,
    })
  }

  pub fn bench(&mut self) -> Result<()> {
    self.clone_remote().context("clone")?;
    self.build_all().context("build all")?;
    self.bench_all().context("bench all")?;

    Ok(())
  }

  fn build_all(&self) -> Result<()> {
    self.cargo_build(&self.local_dir).context("cargo build local")?;
    fs::rename(
      self.local_dir.join("target/release/hvm"),
      self.bin_dir().join("local_hvm"),
    )?;

    for rev in &self.remote_revs {
      let binary = self.bin_dir().join(&rev).join("hvm");

      self.checkout_remote(&rev).with_context(|| format!("checkout {rev}"))?;
      self
        .cargo_build(self.remote_repo_dir())
        .with_context(|| format!("cargo build remote {rev}"))?;

      fs::rename(self.remote_repo_dir().join("target/release/hvm"), &binary)?;
    }

    Ok(())
  }

  fn bench_bin<P: AsRef<Path>>(&mut self, bin: P) -> Result<()> {
    // TODO: clone examples from repo, or use fixed examples
    // run each program on each runtime
    // parse stats

    Ok(())
  }

  fn bench_all(&mut self) -> Result<()> {
    for rev in self.remote_revs.clone() {
      self
        .bench_bin(self.bin_dir().join(&rev).join("hvm"))
        .with_context(|| format!("bench {rev}"))?;
    }

    self
      .bench_bin(self.bin_dir().join("local_hvm"))
      .context("bench local")?;

    Ok(())
  }

  fn remote_repo_dir(&self) -> PathBuf {
    self.tempdir.path().join("repo")
  }

  fn bin_dir(&self) -> PathBuf {
    self.tempdir.path().join("bin")
  }

  fn cargo_build<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    Command::new("cargo")
      .current_dir(dir)
      .args(["build", "--release"])
      .spawn()
      .context("spawn")?
      .wait()
      .context("wait")?;

    Ok(())
  }

  fn clone_remote(&self) -> Result<()> {
    self
      .git()
      .args(["clone", GIT_URL])
      .arg(".")
      .spawn()
      .context("spawn")?
      .wait()
      .context("wait")?;

    Ok(())
  }

  fn checkout_remote(&self, rev: &str) -> Result<()> {
    self
      .git()
      .args(["checkout", rev])
      .spawn()
      .context("spawn")?
      .wait()
      .context("wait")?;

    Ok(())
  }

  fn git(&self) -> Command {
    let mut git = Command::new("git");
    git.current_dir(self.remote_repo_dir());

    git
  }
}
