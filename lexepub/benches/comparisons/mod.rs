mod epub_adapter;
mod epubie_lib_adapter;
mod lexepub_adapter;
mod lib_epub_adapter;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

use self::epub_adapter::EpubAdapter;
use self::epubie_lib_adapter::EpubieLibAdapter;
use self::lexepub_adapter::LexEpubAdapter;
use self::lib_epub_adapter::LibEpubAdapter;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Loading,
    Metadata,
    Extraction,
    Analysis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryTiming {
    pub category: Category,
    pub average_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryResult {
    pub library: String,
    pub timings: Vec<CategoryTiming>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub iterations: usize,
    pub sample_epub: String,
    pub libraries: Vec<LibraryResult>,
}

pub trait Adapter {
    fn name(&self) -> &'static str;
    fn load(&self, path: &Path) -> Result<()>;
    fn metadata(&self, path: &Path) -> Result<()>;
    fn extraction(&self, path: &Path) -> Result<()>;
    fn analysis(&self, path: &Path) -> Result<()>;
}

fn adapters() -> Vec<Box<dyn Adapter>> {
    vec![
        Box::new(LexEpubAdapter),
        Box::new(EpubAdapter),
        Box::new(LibEpubAdapter),
        Box::new(EpubieLibAdapter),
    ]
}

fn sample_candidates() -> [&'static str; 4] {
    [
        "examples/epubs/test-book.epub",
        "examples/epubs/Accessibility-Tests-Extended-Descriptions-v1.1.1.epub",
        "examples/epubs/Fundamental-Accessibility-Tests-Basic-Functionality-v2.0.0.epub",
        "examples/epubs/Fundamental-Accessibility-Tests-Visual-Adjustments-v2.0.0.epub",
    ]
}

pub fn pick_compatible_sample() -> Result<PathBuf> {
    let all_adapters = adapters();
    for candidate in sample_candidates() {
        let path = PathBuf::from(candidate);
        if !path.exists() {
            continue;
        }

        let mut all_ok = true;
        for adapter in &all_adapters {
            if adapter.load(&path).is_err() {
                all_ok = false;
                break;
            }
        }

        if all_ok {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "no compatible sample EPUB found for all configured adapters"
    ))
}

fn measure<F>(iterations: usize, mut op: F) -> Result<f64>
where
    F: FnMut() -> Result<()>,
{
    if iterations == 0 {
        return Err(anyhow!("iterations must be greater than zero"));
    }

    let mut total_ns = 0u128;
    for _ in 0..iterations {
        let start = Instant::now();
        op()?;
        total_ns += start.elapsed().as_nanos();
    }
    // Return milliseconds with nanosecond precision
    Ok(total_ns as f64 / iterations as f64 / 1_000_000.0)
}

pub fn run_comparison(sample_path: &Path, iterations: usize) -> Result<ComparisonReport> {
    let mut libraries = Vec::new();

    for adapter in adapters() {
        let adapter_name = adapter.name();
        let timings = vec![
            CategoryTiming {
                category: Category::Loading,
                average_ms: measure(iterations, || adapter.load(sample_path))
                    .map_err(|e| anyhow!("[{}/Loading] {}", adapter_name, e))?,
            },
            CategoryTiming {
                category: Category::Metadata,
                average_ms: measure(iterations, || adapter.metadata(sample_path))
                    .map_err(|e| anyhow!("[{}/Metadata] {}", adapter_name, e))?,
            },
            CategoryTiming {
                category: Category::Extraction,
                average_ms: measure(iterations, || adapter.extraction(sample_path))
                    .map_err(|e| anyhow!("[{}/Extraction] {}", adapter_name, e))?,
            },
            CategoryTiming {
                category: Category::Analysis,
                average_ms: measure(iterations, || adapter.analysis(sample_path))
                    .map_err(|e| anyhow!("[{}/Analysis] {}", adapter_name, e))?,
            },
        ];

        libraries.push(LibraryResult {
            library: adapter.name().to_string(),
            timings,
        });
    }

    Ok(ComparisonReport {
        iterations,
        sample_epub: sample_path.display().to_string(),
        libraries,
    })
}
