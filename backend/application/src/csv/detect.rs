/// Detects the most likely delimiter from the first non-empty line.
pub fn detect_delimiter(content: &str) -> char {
    let first_line = content.lines().next().unwrap_or("");
    let commas = first_line.matches(',').count();
    let semicolons = first_line.matches(';').count();
    let tabs = first_line.matches('\t').count();

    let mut best = ',';
    let mut best_count = commas;
    if semicolons > best_count {
        best = ';';
        best_count = semicolons;
    }
    if tabs > best_count {
        best = '\t';
    }
    best
}
