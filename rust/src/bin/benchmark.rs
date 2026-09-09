use html_2_json::parse_html;
use std::time::Instant;

fn generate_html_payload(target_bytes: usize) -> String {
    let mut html = String::with_capacity(target_bytes + 1024);
    html.push_str("<!DOCTYPE html><html><head><title>Benchmark Document</title></head><body>");
    html.push_str("<h1>Main Article Title: Benchmark Performance Suite</h1>\n");

    let sample_paragraph = "<p>This is a standard benchmark paragraph with <b>bold text</b>, <i>italic formatting</i>, and <a href=\"https://example.com/test\">a sample hyperlink</a>. It contains multiple sentences to simulate realistic editorial article content.</p>\n";
    let sample_list = "<ul><li>Feature item 1 with <span class=\"highlight\">nested span</span></li><li>Feature item 2 with <code>inline_code()</code></li><li>Feature item 3 with <b>bold metrics</b></li></ul>\n";
    let sample_table = "<table><thead><tr><th>Metric</th><th>Target</th><th>Status</th></tr></thead><tbody><tr><td>Latency</td><td>&lt; 1ms</td><td>Passed</td></tr><tr><td>Throughput</td><td>50 MB/s</td><td>Optimal</td></tr></tbody></table>\n";
    let sample_blockquote = "<blockquote><p>“Performance is an architectural feature, not an afterthought.”</p></blockquote>\n";
    let sample_code = "<pre><code class=\"language-rust\">fn process_data(input: &str) -> Result<ParsedArticle, Error> {\n    parse_html(input)\n}</code></pre>\n";

    let mut section_idx = 1;
    while html.len() < target_bytes {
        html.push_str(&format!("<h2>Section {}: Performance Analysis</h2>\n", section_idx));
        html.push_str(sample_paragraph);
        html.push_str(sample_list);
        html.push_str(sample_blockquote);
        html.push_str(sample_code);
        html.push_str(sample_table);
        section_idx += 1;
    }

    html.push_str("</body></html>");
    html
}

struct BenchmarkResult {
    payload_name: &'static str,
    payload_bytes: usize,
    iterations: usize,
    parse_time_avg_ms: f64,
    json_time_avg_ms: f64,
    total_time_avg_ms: f64,
    throughput_mb_s: f64,
    block_count: usize,
}

fn run_tier(name: &'static str, target_bytes: usize, iterations: usize) -> BenchmarkResult {
    let html = generate_html_payload(target_bytes);
    let actual_bytes = html.len();

    // Warm-up pass
    let warmup_article = parse_html(&html);
    let block_count = warmup_article.len();

    // Measure Parse Time
    let parse_start = Instant::now();
    for _ in 0..iterations {
        let article = parse_html(&html);
        std::hint::black_box(article);
    }
    let parse_duration = parse_start.elapsed();
    let parse_time_avg_ms = (parse_duration.as_secs_f64() * 1000.0) / (iterations as f64);

    // Measure JSON Serialization Time
    let article = parse_html(&html);
    let json_start = Instant::now();
    for _ in 0..iterations {
        let json_str = serde_json::to_string(&article).unwrap();
        std::hint::black_box(json_str);
    }
    let json_duration = json_start.elapsed();
    let json_time_avg_ms = (json_duration.as_secs_f64() * 1000.0) / (iterations as f64);

    let total_time_avg_ms = parse_time_avg_ms + json_time_avg_ms;
    let throughput_mb_s = (actual_bytes as f64 / (1024.0 * 1024.0)) / (total_time_avg_ms / 1000.0);

    BenchmarkResult {
        payload_name: name,
        payload_bytes: actual_bytes,
        iterations,
        parse_time_avg_ms,
        json_time_avg_ms,
        total_time_avg_ms,
        throughput_mb_s,
        block_count,
    }
}

fn main() {
    println!("========================================================================================");
    println!("       ⚡ REACT-NATIVE-FAST-HTML-PARSER EMPIRICAL BENCHMARK SUITE");
    println!("========================================================================================");
    println!("Measuring Native Rust Parsing + 1-Pass JSON Serialization across 6 Payload Tiers\n");

    let tiers = [
        ("1 KB  ", 1 * 1024, 500),
        ("10 KB ", 10 * 1024, 200),
        ("100 KB", 100 * 1024, 50),
        ("500 KB", 500 * 1024, 20),
        ("1 MB  ", 1 * 1024 * 1024, 10),
        ("5 MB  ", 5 * 1024 * 1024, 5),
    ];

    println!("| Payload Tier | Exact Size  | Blocks | Iterations | Parse Time  | JSON Time   | Total Time  | Throughput   |");
    println!("| :----------- | :---------- | :----- | :--------- | :---------- | :---------- | :---------- | :----------- |");

    for (name, bytes, iters) in tiers {
        let res = run_tier(name, bytes, iters);
        println!(
            "| {:<12} | {:>8.2} KB | {:>6} | {:>10} | {:>8.3} ms | {:>8.3} ms | {:>8.3} ms | {:>7.2} MB/s |",
            res.payload_name,
            res.payload_bytes as f64 / 1024.0,
            res.block_count,
            res.iterations,
            res.parse_time_avg_ms,
            res.json_time_avg_ms,
            res.total_time_avg_ms,
            res.throughput_mb_s
        );
    }

    println!("\n========================================================================================");
    println!("✅ Benchmark execution completed successfully.");
    println!("========================================================================================");
}
