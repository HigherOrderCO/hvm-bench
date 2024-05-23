use std::{collections::BTreeMap, fmt::Write};

use anyhow::Result;

use crate::stats::{Program, Stats};

const COLUMN_WIDTH: usize = 14;
const COLUMN_PADDING: &str = "  ";

fn format_header<'a, I: IntoIterator<Item = &'a str>>(revisions: I) -> String {
  let header = vec!["file", "runtime"]
    .into_iter()
    .chain(revisions)
    .map(|col| format!("{col:<COLUMN_WIDTH$}"))
    .collect::<Vec<_>>()
    .join(COLUMN_PADDING);

  format!("{header}\n{}", "=".repeat(header.len()))
}

fn format_rows(stats: &BTreeMap<String, Stats>) -> Result<String> {
  let mut by_program_revision: BTreeMap<String, BTreeMap<String, &Program>> = BTreeMap::new();
  for (revision, programs) in stats {
    for (program, stats) in &programs.programs {
      by_program_revision
        .entry(program.to_string())
        .or_default()
        .insert(revision.to_string(), stats);
    }
  }

  let mut rows = String::new();

  for (program, revisions) in &by_program_revision {
    let interpreted_c = vec![program, "c"]
      .into_iter()
      .chain(
        revisions
          .values()
          .map(|r| r.interpreted_c.as_deref().unwrap_or("error")),
      )
      .map(|col| format!("{col:<COLUMN_WIDTH$}"))
      .collect::<Vec<_>>()
      .join(COLUMN_PADDING);

    let interpreted_cuda = vec![program, "cuda"]
      .into_iter()
      .chain(
        revisions
          .values()
          .map(|r| r.interpreted_cuda.as_deref().unwrap_or("error")),
      )
      .map(|col| format!("{col:<COLUMN_WIDTH$}"))
      .collect::<Vec<_>>()
      .join(COLUMN_PADDING);

    let interpreted_rust = vec![program, "rust"]
      .into_iter()
      .chain(
        revisions
          .values()
          .map(|r| r.interpreted_rust.as_deref().unwrap_or("error")),
      )
      .map(|col| format!("{col:<COLUMN_WIDTH$}"))
      .collect::<Vec<_>>()
      .join(COLUMN_PADDING);

    rows.push_str(&format!("{interpreted_c}\n{interpreted_cuda}\n{interpreted_rust}"));
  }

  Ok(rows)
}

pub fn format(stats: BTreeMap<String, Stats>) -> Result<(String, String)> {
  let mut table = String::new();

  writeln!(table, "interpreted")?;
  writeln!(table, "===========")?;
  writeln!(table);

  writeln!(table, "{}", format_header(stats.keys().map(String::as_str)))?;

  writeln!(table, "{}", format_rows(&stats)?)?;

  Ok((table, "unimplemented".to_string()))
}
