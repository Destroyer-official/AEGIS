use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::scale::{PrefixEntry, ScaleConfig, ScaleMatrixBuilder};

#[derive(Debug, Error)]
pub enum BackfillError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("scale error: {0}")]
    Scale(#[from] crate::scale::ScaleError),
    #[error("invalid date: {0}")]
    InvalidDate(String),
}

/// Output format for backfill results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Csv,
    Json,
    Both,
}

/// Configuration for historical backfill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillConfig {
    pub start_date: String,
    pub end_date: String,
    pub archive_dir: PathBuf,
    pub output_dir: PathBuf,
    pub bgp_file_pattern: String,
    pub seed_file_pattern: String,
    pub output_format: OutputFormat,
    pub scale_config: ScaleConfig,
}

impl Default for BackfillConfig {
    fn default() -> Self {
        Self {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
            archive_dir: PathBuf::from("archives"),
            output_dir: PathBuf::from("output"),
            bgp_file_pattern: "bgp_{date}.csv".to_string(),
            seed_file_pattern: "seeds_{date}.csv".to_string(),
            output_format: OutputFormat::Csv,
            scale_config: ScaleConfig::default(),
        }
    }
}

/// Result of processing a single day.
#[derive(Debug, Clone)]
pub struct DayResult {
    pub date: String,
    pub prefixes: usize,
    pub observations: u64,
    pub elapsed: Duration,
}

/// Aggregate result of the full backfill run.
#[derive(Debug, Clone)]
pub struct BackfillResult {
    pub days_processed: u32,
    pub days_skipped: u32,
    pub total_prefixes: usize,
    pub total_observations: u64,
    pub elapsed: Duration,
    pub per_day: Vec<DayResult>,
}

/// Historical backfill engine that reprocesses date-ranged archives.
pub struct BackfillEngine {
    config: BackfillConfig,
}

impl BackfillEngine {
    pub fn new(config: BackfillConfig) -> Self {
        Self { config }
    }

    /// Run the full backfill across the configured date range.
    pub fn run(&self) -> Result<BackfillResult, BackfillError> {
        let start = Instant::now();
        let dates = date_range(&self.config.start_date, &self.config.end_date)?;

        let mut per_day = Vec::new();
        let mut total_prefixes = 0usize;
        let mut total_observations = 0u64;
        let mut days_skipped = 0u32;

        for date in &dates {
            let (bgp_path, seed_path) = self.resolve_paths(date);

            match self.process_day(date, &bgp_path, &seed_path) {
                Ok(day) => {
                    total_prefixes += day.prefixes;
                    total_observations += day.observations;
                    per_day.push(day);
                }
                Err(e) => {
                    log::warn!("Skipping {}: {}", date, e);
                    days_skipped += 1;
                }
            }
        }

        Ok(BackfillResult {
            days_processed: per_day.len() as u32,
            days_skipped,
            total_prefixes,
            total_observations,
            elapsed: start.elapsed(),
            per_day,
        })
    }

    /// Process a single day of archives.
    fn process_day(
        &self,
        date: &str,
        bgp_path: &Path,
        _seed_path: &Path,
    ) -> Result<DayResult, BackfillError> {
        let start = Instant::now();

        let prefixes = if bgp_path.exists() {
            let file = std::fs::File::open(bgp_path)?;
            let reader = std::io::BufReader::new(file);
            let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(reader);

            let mut prefixes = Vec::new();
            for result in rdr.records() {
                let record = result?;
                if record.len() < 4 {
                    continue;
                }

                let prefix = record[0].trim().to_string();
                let asn: u32 = record[1]
                    .trim()
                    .parse()
                    .map_err(|e: std::num::ParseIntError| {
                        BackfillError::InvalidDate(format!(
                            "invalid ASN '{}': {}",
                            record[1].trim(),
                            e
                        ))
                    })?;
                let as_path: Vec<u32> = record[2]
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                let origin_asn: u32 =
                    record[3]
                        .trim()
                        .parse()
                        .map_err(|e: std::num::ParseIntError| {
                            BackfillError::InvalidDate(format!(
                                "invalid origin ASN '{}': {}",
                                record[3].trim(),
                                e
                            ))
                        })?;
                let roa_valid = if record.len() > 4 {
                    let v = record[4].trim();
                    if v.is_empty() {
                        None
                    } else {
                        v.parse::<bool>().ok()
                    }
                } else {
                    None
                };

                prefixes.push(PrefixEntry {
                    prefix,
                    asn,
                    as_path,
                    origin_asn,
                    roa_valid,
                });
            }
            prefixes
        } else {
            return Err(BackfillError::InvalidDate(format!(
                "no BGP file for {}",
                date
            )));
        };

        let prefix_count = prefixes.len();
        let mut builder = ScaleMatrixBuilder::new(self.config.scale_config.clone());
        let matrix = builder.build_from_prefixes(&prefixes)?;

        Ok(DayResult {
            date: date.to_string(),
            prefixes: prefix_count,
            observations: matrix.total_hosts(),
            elapsed: start.elapsed(),
        })
    }

    fn resolve_paths(&self, date: &str) -> (PathBuf, PathBuf) {
        let bgp = self
            .config
            .archive_dir
            .join(self.config.bgp_file_pattern.replace("{date}", date));
        let seed = self
            .config
            .archive_dir
            .join(self.config.seed_file_pattern.replace("{date}", date));
        (bgp, seed)
    }
}

/// Generate a list of date strings (YYYY-MM-DD) between start and end inclusive.
fn date_range(start: &str, end: &str) -> Result<Vec<String>, BackfillError> {
    let start = parse_date(start)?;
    let end = parse_date(end)?;

    let mut dates = Vec::new();
    let mut current = start;
    while current <= end {
        dates.push(format_date(current));
        current += 1;
    }
    Ok(dates)
}

/// Parse YYYY-MM-DD to days since epoch.
fn parse_date(s: &str) -> Result<u32, BackfillError> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(BackfillError::InvalidDate(s.to_string()));
    }
    let year: u32 = parts[0]
        .parse()
        .map_err(|_| BackfillError::InvalidDate(s.to_string()))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| BackfillError::InvalidDate(s.to_string()))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| BackfillError::InvalidDate(s.to_string()))?;

    if !(1..=12).contains(&month) {
        return Err(BackfillError::InvalidDate(format!(
            "month {} out of range 1-12 in '{}'",
            month, s
        )));
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    };
    if day < 1 || day > max_day {
        return Err(BackfillError::InvalidDate(format!(
            "day {} out of range 1-{} for month {} in '{}'",
            day, max_day, month, s
        )));
    }

    let month_days = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut days = 0u32;
    // Count days from 1970-01-01 to year-01-01.
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    // Count days for completed months in the current year.
    for m in 0..(month - 1) {
        let feb = if is_leap(year) { 29 } else { 28 };
        let md = match m {
            1 => feb,
            _ => month_days[m as usize],
        };
        days += md;
    }
    // Add days in current month (1-indexed to 0-indexed).
    days += day - 1;
    Ok(days)
}

/// Format days-since-epoch back to YYYY-MM-DD.
fn format_date(days: u32) -> String {
    let mut y = 1970u32;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let leap = is_leap(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0u32;
    while m < 12 && remaining >= month_days[m as usize] {
        remaining -= month_days[m as usize];
        m += 1;
    }

    format!("{:04}-{:02}-{:02}", y, m + 1, remaining + 1)
}

fn is_leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_date_range() {
        let dates = date_range("2024-01-01", "2024-01-05").unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], "2024-01-01");
        assert_eq!(dates[4], "2024-01-05");
    }

    #[test]
    fn test_date_range_month_boundary() {
        let dates = date_range("2024-01-29", "2024-02-02").unwrap();
        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0], "2024-01-29");
        assert_eq!(dates[2], "2024-01-31");
        assert_eq!(dates[3], "2024-02-01");
    }

    #[test]
    fn test_parse_format_roundtrip() {
        let days = parse_date("2024-06-15").unwrap();
        let formatted = format_date(days);
        assert_eq!(formatted, "2024-06-15");
    }

    #[test]
    fn test_resolve_paths() {
        let config = BackfillConfig {
            archive_dir: PathBuf::from("/data"),
            bgp_file_pattern: "bgp_{date}.csv".to_string(),
            seed_file_pattern: "seeds_{date}.csv".to_string(),
            ..Default::default()
        };
        let engine = BackfillEngine::new(config);
        let (bgp, seed) = engine.resolve_paths("2024-03-15");
        assert_eq!(bgp, PathBuf::from("/data/bgp_2024-03-15.csv"));
        assert_eq!(seed, PathBuf::from("/data/seeds_2024-03-15.csv"));
    }

    #[test]
    fn test_process_day_missing_file() {
        let config = BackfillConfig {
            archive_dir: PathBuf::from("/nonexistent"),
            ..Default::default()
        };
        let engine = BackfillEngine::new(config);
        let result = engine.process_day(
            "2024-01-01",
            &PathBuf::from("/nonexistent/bgp_2024-01-01.csv"),
            &PathBuf::from("/nonexistent/seeds_2024-01-01.csv"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_day_with_data() {
        let tmp = std::env::temp_dir().join("aegis_backfill_test");
        std::fs::create_dir_all(&tmp).unwrap();

        let bgp_path = tmp.join("bgp_2024-01-01.csv");
        let mut f = std::fs::File::create(&bgp_path).unwrap();
        writeln!(
            f,
            "prefix,asn,as_path,origin_asn,roa_valid\n10.0.0.0/8,16509,16509,16509,true\n192.168.0.0/16,15169,15169,15169,false"
        )
        .unwrap();

        let config = BackfillConfig {
            archive_dir: tmp.clone(),
            ..Default::default()
        };
        let engine = BackfillEngine::new(config);
        let result = engine
            .process_day("2024-01-01", &bgp_path, Path::new(""))
            .unwrap();

        assert_eq!(result.date, "2024-01-01");
        assert_eq!(result.prefixes, 2);
        assert!(result.observations > 0);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_backfill_result_totals() {
        let tmp = std::env::temp_dir().join("aegis_backfill_test2");
        std::fs::create_dir_all(&tmp).unwrap();

        // Create 3 days of data.
        for day in 1..=3 {
            let bgp_path = tmp.join(format!("bgp_2024-01-{:02}.csv", day));
            let mut f = std::fs::File::create(&bgp_path).unwrap();
            writeln!(
                f,
                "prefix,asn,as_path,origin_asn,roa_valid\n10.{}.0.0/8,16509,16509,16509,true",
                day
            )
            .unwrap();
        }

        let config = BackfillConfig {
            archive_dir: tmp.clone(),
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-03".to_string(),
            ..Default::default()
        };
        let engine = BackfillEngine::new(config);
        let result = engine.run().unwrap();

        assert_eq!(result.days_processed, 3);
        assert_eq!(result.total_prefixes, 3);
        assert!(result.total_observations > 0);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_is_leap() {
        assert!(is_leap(2000));
        assert!(!is_leap(1900));
        assert!(is_leap(2024));
        assert!(!is_leap(2023));
    }

    #[test]
    fn test_parse_date_validation() {
        // Valid dates
        assert!(parse_date("2024-01-01").is_ok());
        assert!(parse_date("2024-12-31").is_ok());
        assert!(parse_date("2024-02-29").is_ok()); // leap year

        // Invalid month
        assert!(parse_date("2024-00-01").is_err());
        assert!(parse_date("2024-13-01").is_err());

        // Invalid day
        assert!(parse_date("2024-01-00").is_err());
        assert!(parse_date("2024-01-32").is_err());

        // Invalid day for month
        assert!(parse_date("2024-02-30").is_err()); // Feb max 29 in leap year
        assert!(parse_date("2023-02-29").is_err()); // Feb max 28 in non-leap year
        assert!(parse_date("2024-04-31").is_err()); // Apr max 30

        // Invalid format
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("2024/01/01").is_err());
    }
}
