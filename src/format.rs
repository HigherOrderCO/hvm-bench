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

fn format_compiled_rows(stats: &BTreeMap<String, Stats>) -> Result<String> {
  macro_rules! writeln_runtime {
    ($rows:ident, $revisions:ident, $program:expr, $runtime:expr) => {{
      let row = vec![$program, $runtime]
        .into_iter()
        .chain(
          $revisions
            .values()
            .map(|r| r.compiled($runtime).as_deref().unwrap_or("error")),
        )
        .enumerate()
        .map(|(i, col)| {
          if i < 2 {
            format!("{col:<COLUMN_WIDTH$}")
          } else {
            format!("{col:>COLUMN_WIDTH$}")
          }
        })
        .collect::<Vec<_>>()
        .join(COLUMN_PADDING);

      writeln!($rows, "{row}")?;

      row
    }};
  }

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
    writeln_runtime!(rows, revisions, program, "c");
    let row = writeln_runtime!(rows, revisions, "", "cuda");

    writeln!(rows, "{}", "-".repeat(row.len()))?;
  }

  Ok(rows)
}

fn format_interpreted_rows(stats: &BTreeMap<String, Stats>) -> Result<String> {
  macro_rules! writeln_runtime {
    ($rows:ident, $revisions:ident, $program:expr, $runtime:expr) => {{
      let row = vec![$program, $runtime]
        .into_iter()
        .chain(
          $revisions
            .values()
            .map(|r| r.interpreted($runtime).as_deref().unwrap_or("error")),
        )
        .enumerate()
        .map(|(i, col)| {
          if i < 2 {
            format!("{col:<COLUMN_WIDTH$}")
          } else {
            format!("{col:>COLUMN_WIDTH$}")
          }
        })
        .collect::<Vec<_>>()
        .join(COLUMN_PADDING);

      writeln!($rows, "{row}")?;

      row
    }};
  }

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
    writeln_runtime!(rows, revisions, program, "c");
    writeln_runtime!(rows, revisions, "", "cuda");
    let row = writeln_runtime!(rows, revisions, "", "rust");

    writeln!(rows, "{}", "-".repeat(row.len()))?;
  }

  Ok(rows)
}

pub fn format(stats: &BTreeMap<String, Stats>) -> Result<String> {
  let mut table = String::new();

  writeln!(table, "compiled")?;
  writeln!(table, "========")?;
  writeln!(table)?;

  writeln!(table, "{}", format_header(stats.keys().map(String::as_str)))?;
  writeln!(table, "{}", format_compiled_rows(stats)?)?;

  writeln!(table, "interpreted")?;
  writeln!(table, "===========")?;
  writeln!(table)?;

  writeln!(table, "{}", format_header(stats.keys().map(String::as_str)))?;
  writeln!(table, "{}", format_interpreted_rows(stats)?)?;

  Ok(table)
}
