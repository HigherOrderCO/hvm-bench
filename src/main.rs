mod bench;
mod stats;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use self::bench::Bench;

#[derive(Parser)]
struct Args {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  Bench {
    /// Path to local hvm repo to benchmark.
    #[arg(long, default_value = "./hvm")]
    repo_dir: PathBuf,
    /// Path to output benchmarking results.
    #[arg(long, default_value = "./out")]
    out_dir: PathBuf,
    /// Which revisions in the remote repository to benchmark.
    #[arg(short, long)]
    revs: Vec<String>,
  },
}

fn main() -> Result<()> {
  match Args::parse().command {
    Command::Bench {
      repo_dir,
      out_dir,
      revs,
    } => {
      if !repo_dir.exists() {
        anyhow::bail!("{repo_dir:?} does not exist");
      }

      let bench = Bench::new(repo_dir, revs);

      if !out_dir.exists() {
        fs::create_dir_all(out_dir).context("create_dir_all")?;
      }
    }
  }

  Ok(())
}
