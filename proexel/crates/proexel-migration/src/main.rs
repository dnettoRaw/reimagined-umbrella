use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use proexel_infrastructure::JsonFileStore;
use proexel_migration::{migrate_bundle, LegacyBundle, MigrationReport};

const MAX_LEGACY_INPUT_BYTES: u64 = 64 * 1024 * 1024;

struct Options {
    input: PathBuf,
    state: PathBuf,
    batch: String,
    report_json: Option<PathBuf>,
    report_markdown: Option<PathBuf>,
    dry_run: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("migration failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_options(std::env::args().skip(1).collect())?;
    let bytes = read_input(&options.input)?;
    let bundle: LegacyBundle =
        serde_json::from_slice(&bytes).map_err(|error| format!("input JSON invalid: {error}"))?;
    let store = JsonFileStore::new(&options.state)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock invalid".to_string())?
        .as_millis() as u64;
    let report = if options.dry_run {
        let mut state = store.read()?;
        migrate_bundle(&bundle, &mut state, &options.batch, now, true)?
    } else {
        store.transact(|state| migrate_bundle(&bundle, state, &options.batch, now, false))?
    };
    let json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    println!("{json}");
    if let Some(path) = options.report_json {
        write_report(&path, json.as_bytes())?;
    }
    if let Some(path) = options.report_markdown {
        write_report(&path, markdown_report(&report).as_bytes())?;
    }
    Ok(())
}

fn read_input(path: &PathBuf) -> Result<Vec<u8>, String> {
    let size = fs::metadata(path)
        .map_err(|error| format!("input metadata failed: {error}"))?
        .len();
    if size > MAX_LEGACY_INPUT_BYTES {
        return Err(format!(
            "input too large: {size} > {MAX_LEGACY_INPUT_BYTES}"
        ));
    }
    fs::read(path).map_err(|error| format!("input read failed: {error}"))
}

fn parse_options(args: Vec<String>) -> Result<Options, String> {
    let mut input = None;
    let mut state = None;
    let mut batch = None;
    let mut report_json = None;
    let mut report_markdown = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => input = Some(next_value(&args, &mut index, "--input")?.into()),
            "--state" => state = Some(next_value(&args, &mut index, "--state")?.into()),
            "--batch" => batch = Some(next_value(&args, &mut index, "--batch")?),
            "--report-json" => {
                report_json = Some(next_value(&args, &mut index, "--report-json")?.into())
            }
            "--report-markdown" => {
                report_markdown = Some(next_value(&args, &mut index, "--report-markdown")?.into())
            }
            "--dry-run" => dry_run = true,
            "--help" | "-h" => return Err(usage()),
            value => return Err(format!("unknown option {value}\n{}", usage())),
        }
        index += 1;
    }
    Ok(Options {
        input: input.ok_or_else(usage)?,
        state: state.ok_or_else(usage)?,
        batch: batch.ok_or_else(usage)?,
        report_json,
        report_markdown,
        dry_run,
    })
}

fn next_value(args: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {option}"))
}

fn write_report(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("report directory failed: {error}"))?;
    }
    fs::write(path, bytes).map_err(|error| format!("report write failed: {error}"))
}

fn markdown_report(report: &MigrationReport) -> String {
    let mut output = format!(
        "# PROEXEL migration report\n\n- Batch: `{}`\n- Checksum: `{}`\n- Dry-run: `{}`\n\n## Counts\n\n| Entity | Source | Imported |\n|---|---:|---:|\n",
        report.batch_id, report.checksum, report.dry_run
    );
    for (entity, source) in &report.source_counts {
        let imported = report.imported_counts.get(entity).copied().unwrap_or(0);
        output.push_str(&format!("| {entity} | {source} | {imported} |\n"));
    }
    output.push_str("\n## Warnings\n\n");
    if report.warnings.is_empty() {
        output.push_str("None.\n");
    } else {
        for warning in &report.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
    }
    output
}

fn usage() -> String {
    "usage: proexel-migrate --input <legacy.json> --state <state.json> --batch <id> [--dry-run] [--report-json <path>] [--report-markdown <path>]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_input_is_rejected_before_reading_payload() {
        let path = std::env::temp_dir().join(format!(
            "proexel-migration-oversized-{}.json",
            std::process::id()
        ));
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_LEGACY_INPUT_BYTES + 1).unwrap();

        let result = read_input(&path);

        assert!(result
            .err()
            .is_some_and(|error| error.starts_with("input too large:")));
        let _ = fs::remove_file(path);
    }
}
