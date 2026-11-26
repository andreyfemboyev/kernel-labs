use serde::Serialize;

#[derive(Serialize)]
pub struct Finding<'a> {
    line: usize,
    timestamp_seconds: Option<f64>,
    heuristic: &'static str,
    text: &'a str,
}

pub fn analyze(text: &str) -> Vec<Finding<'_>> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let lower = line.to_ascii_lowercase();
            let kind = [
                ("kernel panic", "panic"),
                ("out of memory", "out_of_memory"),
                ("oom-kill:", "out_of_memory"),
                ("oops:", "oops"),
                ("warning:", "warning"),
                ("unknown symbol", "module_symbol"),
                ("call trace:", "trace"),
            ]
            .into_iter()
            .find(|(marker, _)| lower.contains(marker))?
            .1;
            let timestamp_seconds = line
                .trim_start()
                .strip_prefix('[')
                .and_then(|s| s.split_once(']'))
                .and_then(|(value, _)| value.trim().parse::<f64>().ok())
                .filter(|value| value.is_finite());
            Some(Finding {
                line: i + 1,
                timestamp_seconds,
                heuristic: kind,
                text: line,
            })
        })
        .collect()
}
