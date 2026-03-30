use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fu::cli::Options;
use fu::data;

fn generate_hist_lines(n: usize) -> Vec<String> {
    // Deterministic pseudo-random: LCG producing gaussian-ish values
    let mut state: u64 = 42;
    let mut lines = Vec::with_capacity(n);
    for _ in 0..n {
        // Box-Muller-ish: sum 6 uniforms for ~normal distribution
        let mut sum: f64 = 0.0;
        for _ in 0..6 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            sum += (state >> 33) as f64 / (1u64 << 31) as f64;
        }
        let val = (sum - 3.0) * 15.0 + 50.0;
        lines.push(format!("{val:.6}"));
    }
    lines
}

fn generate_hist_text(n: usize) -> String {
    generate_hist_lines(n).join("\n")
}

fn bench_parse_hist(c: &mut Criterion) {
    let mut group = c.benchmark_group("hist_parse");

    for &n in &[10_000, 100_000, 1_000_000] {
        let lines = generate_hist_lines(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("parse_hist_lines", n), &lines, |b, lines| {
            b.iter(|| {
                let mut values = Vec::with_capacity(lines.len());
                for line in lines.iter().filter(|l| !l.is_empty()) {
                    let field = line.split('\t').next().unwrap_or("").trim();
                    let v: f64 = field.parse().unwrap();
                    values.push(v);
                }
                values
            });
        });
    }

    group.finish();
}

fn bench_bin_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("hist_bin");

    for &n in &[10_000, 100_000, 1_000_000] {
        let lines = generate_hist_lines(n);
        let values: Vec<f64> = lines
            .iter()
            .map(|l| l.parse::<f64>().unwrap())
            .collect();

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("bin_values", n), &values, |b, values| {
            b.iter(|| data::bin_values(values, 10));
        });
        group.bench_with_input(
            BenchmarkId::new("bin_values_log", n),
            &values,
            |b, values| {
                b.iter(|| data::bin_values_log(values, 10).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_filter_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("hist_filter");

    for &n in &[10_000, 100_000, 1_000_000] {
        let lines = generate_hist_lines(n);
        let values: Vec<f64> = lines
            .iter()
            .map(|l| l.parse::<f64>().unwrap())
            .collect();

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::new("filter_gt_lt", n),
            &values,
            |b, values| {
                b.iter(|| data::filter_values(values.clone(), Some(30.0), Some(70.0)));
            },
        );
    }

    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("hist_e2e");

    for &n in &[10_000, 100_000, 1_000_000] {
        let text = generate_hist_text(n);

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::new("hist_pipeline", n),
            &text,
            |b, text| {
                b.iter(|| {
                    // Simulate the full pipeline: split lines, parse, bin
                    let lines: Vec<String> = text.lines().map(String::from).collect();
                    let mut values = Vec::with_capacity(lines.len());
                    for line in lines.iter().filter(|l| !l.is_empty()) {
                        let field = line.split('\t').next().unwrap_or("").trim();
                        let v: f64 = field.parse().unwrap();
                        values.push(v);
                    }
                    data::bin_values(&values, 10)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_hist,
    bench_bin_values,
    bench_filter_values,
    bench_end_to_end,
);
criterion_main!(benches);
