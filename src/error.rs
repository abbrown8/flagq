// Every error that can reach the user carries a 1-based line and column so
// it can be pointed at directly in the source file, the same way a
// compiler error would be. `render` is the one place that turns a raw
// (line, col, message) into the boxed, arrow-and-caret text people
// actually read.

pub struct SourceError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl SourceError {
    pub fn new(line: usize, col: usize, message: impl Into<String>) -> Self {
        SourceError {
            line,
            col,
            message: message.into(),
        }
    }

    pub fn render(&self, filename: &str, source: &str) -> String {
        let line_text = source.lines().nth(self.line.saturating_sub(1)).unwrap_or("");
        let gutter = format!("{}", self.line);
        let pad = " ".repeat(gutter.len());
        let caret_offset = self.col.saturating_sub(1);
        let caret_line: String = line_text
            .chars()
            .take(caret_offset)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();

        format!(
            "error: {msg}\n {pad}--> {file}:{line}:{col}\n {pad} |\n {gut} | {text}\n {pad} | {caret}^\n",
            msg = self.message,
            pad = pad,
            file = filename,
            line = self.line,
            col = self.col,
            gut = gutter,
            text = line_text,
            caret = caret_line,
        )
    }
}

/// Cheap edit distance, used only to suggest "did you mean" on typos in
/// flag names and context keys. Good enough at the short identifier
/// lengths this tool deals with; no need for anything fancier.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();

    for i in 1..=a.len() {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let cur = row[j];
            row[j] = if a[i - 1] == b[j - 1] {
                prev
            } else {
                1 + prev.min(row[j]).min(row[j - 1])
            };
            prev = cur;
        }
    }
    row[b.len()]
}

/// Finds the closest candidate to `target`, if any is within a distance
/// that's plausibly a typo rather than a different word entirely.
pub fn suggest<'a>(target: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut best: Option<(&str, usize)> = None;
    for candidate in candidates {
        let dist = levenshtein(target, candidate);
        if dist <= 2 && best.map_or(true, |(_, best_dist)| dist < best_dist) {
            best = Some((candidate, dist));
        }
    }
    best.map(|(name, _)| name)
}
