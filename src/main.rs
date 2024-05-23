mod bench;
mod format;
mod run;
mod stats;

use std::{path::PathBuf, time::Duration};

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
    /// Which revisions in the remote repository to benchmark.
    #[arg(short, long)]
    revs: Vec<String>,
    /// Timeout in seconds
    #[arg(long, default_value_t = 60)]
    timeout: u64,
  },
}

fn main() -> Result<()> {
  match Args::parse().command {
    Command::Bench {
      repo_dir,
      revs,
      timeout,
    } => {
      if !repo_dir.exists() {
        anyhow::bail!("{repo_dir:?} does not exist");
      }

      let mut bench = Bench::new(repo_dir, revs, Duration::from_secs(timeout)).context("Bench::new")?;
      bench.bench().context("bench")?;

      println!("{}", format::format(&bench.stats).context("format")?);
    }
  }

  Ok(())
}
