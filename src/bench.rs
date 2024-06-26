use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
  time::Duration,
};

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::{
  ext::CommandExt,
  run,
  stats::{Program, Stats},
};

const GIT_URL: &str = "https://github.com/HigherOrderCO/hvm.git";
const PROGRAMS_DIR: &str = "./programs";

pub struct Bench {
  /// Local hvm directory.
  local_dir: PathBuf,
  /// Remote revisions.
  remote_revs: Vec<String>,
  /// Timeout for runs.
  timeout: Duration,
  /// Statistics collected for each revision.
  pub stats: BTreeMap<String, Stats>,
  /// Temporary directory for binaries and remote repo.
  tempdir: TempDir,
}

impl Bench {
  pub fn new(local_dir: PathBuf, remote_revs: Vec<String>, timeout: Duration) -> Result<Self> {
    let tempdir = TempDir::with_prefix("hvm-bench-").context("tempdir")?;

    fs::create_dir(tempdir.path().join("repo")).context("create_dir repo")?;
    fs::create_dir(tempdir.path().join("bin")).context("create_dir bin")?;

    Ok(Self {
      local_dir,
      remote_revs,
      timeout,
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
    )
    .context("rename local")?;

    for rev in &self.remote_revs {
      let bin_rev_dir = self.bin_dir().join(rev);
      fs::create_dir(&bin_rev_dir).context("create dir")?;

      let binary = bin_rev_dir.join("hvm");

      self.checkout_remote(rev).with_context(|| format!("checkout {rev}"))?;
      self
        .cargo_build(self.remote_repo_dir())
        .with_context(|| format!("cargo build remote {rev}"))?;

      fs::rename(self.remote_repo_dir().join("target/release/hvm"), &binary).context("rename remote")?;
    }

    Ok(())
  }

  fn bench_all(&mut self) -> Result<()> {
    for rev in self.remote_revs.clone() {
      self
        .bench_bin(&rev, self.bin_dir().join(&rev).join("hvm"))
        .with_context(|| format!("bench {rev}"))?;
    }

    self
      .bench_bin("(local)", self.bin_dir().join("local_hvm"))
      .context("bench local")?;

    Ok(())
  }

  fn bench_bin<P: AsRef<Path>>(&mut self, rev: &str, bin: P) -> Result<()> {
    eprintln!("benchmarking {rev:?}");
    for program in self.programs().context("programs")? {
      eprintln!("  running {program:?}");

      let program_name = program.file_stem().context("file stem")?.to_string_lossy().into_owned();

      self.stats.entry(rev.to_string()).or_default().programs.insert(
        program_name,
        Program {
          interpreted_c: run::interpreted_c(&bin, &program, self.timeout),
          interpreted_cuda: run::interpreted_cuda(&bin, &program, self.timeout),
          interpreted_rust: run::interpreted_rust(&bin, &program, self.timeout),
          compiled_c: Ok(run::compiled_c(&bin, &program, self.timeout).unwrap()),
          compiled_cuda: run::compiled_cuda(&bin, &program, self.timeout),
        },
      );
    }

    Ok(())
  }

  fn programs(&self) -> Result<Vec<PathBuf>> {
    fs::read_dir(PROGRAMS_DIR)
      .context("read dir")?
      .map(|entry| Ok(entry?.path()))
      .collect()
  }

  fn remote_repo_dir(&self) -> PathBuf {
    self.tempdir.path().join("repo")
  }

  fn bin_dir(&self) -> PathBuf {
    self.tempdir.path().join("bin")
  }

  fn cargo_build<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    eprintln!("building {dir:?}", dir = dir.as_ref());

    Command::new("cargo")
      .current_dir(dir)
      .args(["build", "--release"])
      .status_stdout()
      .context("status stdout")?;

    Ok(())
  }

  fn clone_remote(&self) -> Result<()> {
    self
      .git()
      .args(["clone", GIT_URL])
      .arg(".")
      .status_stdout()
      .context("status stdout")?;

    Ok(())
  }

  fn checkout_remote(&self, rev: &str) -> Result<()> {
    self
      .git()
      .args(["checkout", rev])
      .status_stdout()
      .context("status stdout")?;

    Ok(())
  }

  fn git(&self) -> Command {
    let mut git = Command::new("git");
    git.current_dir(self.remote_repo_dir());

    git
  }
}
