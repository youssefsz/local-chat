//! Dedicated synthetic search benchmark, excluded from distribution.
use local_chat::storage::Store;
use std::time::Instant;
fn main() -> anyhow::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .expect("isolated output directory required");
    std::fs::create_dir_all(&dir)?;
    let path = std::path::Path::new(&dir).join("benchmark.sqlite3");
    anyhow::ensure!(!path.exists(), "Use a fresh benchmark directory");
    let mut db = Store::open(&path)?;
    let context = db.activate(Some("synthetic-performance-fixture"), "Synthetic benchmark")?;
    let start = Instant::now();
    db.seed_search_fixture(100_000)?;
    let seed_ms = start.elapsed().as_secs_f64() * 1000.;
    let mut times = Vec::new();
    for i in 0..200 {
        let start = Instant::now();
        let result = db.search(
            if i % 2 == 0 {
                "needle"
            } else {
                "local message"
            },
            None,
        )?;
        assert!(!result.is_empty());
        times.push(start.elapsed().as_secs_f64() * 1000.);
    }
    times.sort_by(f64::total_cmp);
    let mut pages = Vec::new();
    for _ in 0..200 {
        let start = Instant::now();
        assert_eq!(db.messages(&context, "general", None, None)?.len(), 80);
        pages.push(start.elapsed().as_secs_f64() * 1000.);
    }
    pages.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::json!({"dataset":100000,"search_runs":200,"search_p50_ms":times[100],"search_p95_ms":times[190],"message_page_p95_ms":pages[190],"seed_ms":seed_ms,"build":"release","fixture":"synthetic text, 1000 needle matches, 99000 common matches; same SQLite schema and FTS triggers"})
    );
    Ok(())
}
