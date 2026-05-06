mod comparisons;

use anyhow::{Context, Result};
use clap::Parser;
use comparisons::{
    pick_compatible_sample, run_comparison, Category, ComparisonReport, LibraryResult,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Terminal,
};
use std::{
    collections::HashMap,
    fs,
    io::{self, Stdout},
    path::PathBuf,
    time::Instant,
};

// CLI
#[derive(Parser, Debug)]
#[command(
    name = "comparison",
    about = "Benchmark EPUB library performance",
    version
)]
struct Args {
    /// Number of benchmark iterations per library
    #[arg(long, default_value_t = 20)]
    iterations: usize,

    /// Path to write the JSON report
    #[arg(long)]
    output: Option<PathBuf>,

    /// Launch the interactive TUI after benchmarking
    #[arg(long)]
    tui: bool,

    /// Path to a specific EPUB file to use as the sample
    #[arg(long)]
    epub: Option<PathBuf>,

    /// Print a summary table to stdout (default when --tui is not set)
    #[arg(long)]
    print: bool,
}

//  Data helpers
/// Build a category -> average_ms lookup for one library.
fn timing_map(result: &LibraryResult) -> HashMap<Category, f64> {
    result
        .timings
        .iter()
        .map(|t| (t.category, t.average_ms))
        .collect()
}

/// Return the best (lowest) average for a category across all libraries.
fn best_for(report: &ComparisonReport, category: Category) -> f64 {
    report
        .libraries
        .iter()
        .filter_map(|lib| timing_map(lib).get(&category).copied())
        .fold(f64::MAX, f64::min)
}

/// Format a millisecond value with a relative indicator vs. the best performer.
fn fmt_ms(ms: f64, best: f64) -> String {
    let ratio = ms / best;
    if ratio < 1.05 {
        format!("{ms:.3} ms ✓")
    } else {
        format!("{ms:.3} ms ({ratio:.1}x)")
    }
}

// Terminal helpers
struct TerminalGuard(Terminal<CrosstermBackend<Stdout>>);

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode().context("enable raw mode")?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self(terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort cleanup: ignore errors during drop.
        let _ = disable_raw_mode();
        let _ = self.0.backend_mut().execute(LeaveAlternateScreen);
        let _ = self.0.show_cursor();
    }
}

// TUI
const CATEGORIES: &[Category] = &[
    Category::Loading,
    Category::Metadata,
    Category::Extraction,
    Category::Analysis,
];

const CATEGORY_LABELS: &[&str] = &[
    "Loading (ms)",
    "Metadata (ms)",
    "Extraction (ms)",
    "Analysis (ms)",
];

fn render_tui(report: &ComparisonReport) -> Result<()> {
    let mut guard = TerminalGuard::enter()?;
    let terminal = &mut guard.0;

    // Pre-compute best times so we don't recalculate every frame.
    let bests: Vec<f64> = CATEGORIES
        .iter()
        .map(|&cat| best_for(report, cat))
        .collect();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // header
                    Constraint::Min(6),    // table
                    Constraint::Length(5), // sparklines / gauges
                    Constraint::Length(3), // footer
                ])
                .split(area);

            //  Header
            let header_text = format!(
                " EPUB Benchmark  │  sample: {}  │  iterations: {}",
                report.sample_epub, report.iterations
            );
            f.render_widget(
                Paragraph::new(header_text)
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL)),
                chunks[0],
            );

            //  Results table
            let col_header = std::iter::once("Library")
                .chain(CATEGORY_LABELS.iter().copied())
                .map(|s| {
                    Cell::from(s).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                });
            let header_row = Row::new(col_header).height(1);

            let rows: Vec<Row> = report
                .libraries
                .iter()
                .map(|lib| {
                    let m = timing_map(lib);
                    let cells: Vec<Cell> = std::iter::once(Cell::from(lib.library.clone()))
                        .chain(CATEGORIES.iter().zip(&bests).map(|(&cat, &best)| {
                            let ms = m.get(&cat).copied().unwrap_or(0.0);
                            let is_best = ms <= best * 1.05;
                            Cell::from(fmt_ms(ms, best)).style(if is_best {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::White)
                            })
                        }))
                        .collect();
                    Row::new(cells)
                })
                .collect();

            let widths = [
                Constraint::Length(16),
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Length(20),
                Constraint::Length(18),
            ];
            let table = Table::new(rows, widths)
                .header(header_row)
                .block(Block::default().title(" Results ").borders(Borders::ALL))
                .column_spacing(1);
            f.render_widget(table, chunks[1]);

            //  Loading-time gauge (relative to slowest)
            let gauge_area = chunks[2];
            let gauge_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1); report.libraries.len() + 1])
                .split(gauge_area);

            let worst_loading = report
                .libraries
                .iter()
                .filter_map(|l| timing_map(l).get(&Category::Loading).copied())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(title_area) = gauge_chunks.first() {
                f.render_widget(
                    Paragraph::new(" Loading time (relative) ")
                        .style(Style::default().fg(Color::Yellow)),
                    *title_area,
                );
            }
            if let Some(worst_ms) = worst_loading {
                for (i, lib) in report.libraries.iter().enumerate() {
                    if let Some(&area) = gauge_chunks.get(i + 1) {
                        if let Some(ms) = timing_map(lib)
                            .get(&Category::Loading)
                            .copied()
                        {
                            let ratio = if worst_ms > 0.0 {
                                (ms / worst_ms * 100.0) as u16
                            } else {
                                100
                            };
                            let gauge = Gauge::default()
                                .label(format!("{} {:.3} ms", lib.library, ms))
                                .percent(ratio)
                                .gauge_style(Style::default().fg(Color::Blue));
                            f.render_widget(gauge, area);
                        }
                    }
                }
            }

            // Footer
            f.render_widget(
                Paragraph::new(" [q] quit  │  ✓ = best performer  │  (Nx) = N× slower than best")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL)),
                chunks[3],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(()) // TerminalGuard::drop handles cleanup
}

// Stdout summary
fn print_summary(report: &ComparisonReport) {
    let bests: Vec<f64> = CATEGORIES
        .iter()
        .map(|&cat| best_for(report, cat))
        .collect();

    println!(
        "\nEPUB Benchmark | sample: {} | {} iterations\n",
        report.sample_epub, report.iterations
    );

    let label_width = report
        .libraries
        .iter()
        .map(|l| l.library.len())
        .max()
        .unwrap_or(8)
        .max(8);

    print!("{:<width$}", "Library", width = label_width + 2);
    for label in CATEGORY_LABELS {
        print!("  {:<20}", label);
    }
    println!();
    println!(
        "{}",
        "-".repeat(label_width + 2 + CATEGORY_LABELS.len() * 22)
    );

    for lib in &report.libraries {
        let m = timing_map(lib);
        print!("{:<width$}", lib.library, width = label_width + 2);
        for (&cat, &best) in CATEGORIES.iter().zip(&bests) {
            let ms = m.get(&cat).copied().unwrap_or(0.0);
            print!("  {:<20}", fmt_ms(ms, best));
        }
        println!();
    }
    println!();
}

// Entry point
fn main() -> Result<()> {
    // Strip `--bench` injected by `cargo bench` so Clap doesn't choke on it.
    let filtered_args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    let args = Args::parse_from(filtered_args);

    let output_path = if let Some(path) = args.output {
        path
    } else {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(manifest_dir).join("target/comparisons/latest.json")
    };

    let sample = match args.epub {
        Some(path) => {
            anyhow::ensure!(path.exists(), "EPUB not found: {}", path.display());
            path
        }
        None => pick_compatible_sample().context("locating a compatible sample EPUB")?,
    };

    println!(
        "▶ Running {} iteration(s) on {} …",
        args.iterations,
        sample.display()
    );
    let start = Instant::now();
    let report = run_comparison(&sample, args.iterations).context("running comparison")?;
    println!("✓ Done in {:.2?}", start.elapsed());

    // Persist JSON report.
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&report).context("serialising report")?;
    fs::write(&output_path, &json)
        .with_context(|| format!("writing report to {}", output_path.display()))?;
    println!("Report saved -> {}", output_path.display());

    // Output.
    if args.tui {
        render_tui(&report)?;
    } else {
        print_summary(&report);
    }

    Ok(())
}
