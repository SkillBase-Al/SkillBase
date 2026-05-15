use std::collections::HashSet;
use crate::db::models::InstalledSkill;

/// Normalize a name for comparison: lowercase, strip hyphens/underscores, remove non-alphanumeric
fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Compute Levenshtein distance between two strings
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(curr_row[j] + 1, prev_row[j + 1] + 1),
                prev_row[j] + cost,
            );
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Compute Jaccard similarity of character bigrams
fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let bigrams_a: HashSet<(char, char)> = a
        .chars()
        .zip(a.chars().skip(1))
        .collect();
    let bigrams_b: HashSet<(char, char)> = b
        .chars()
        .zip(b.chars().skip(1))
        .collect();

    if bigrams_a.is_empty() && bigrams_b.is_empty() {
        return 1.0;
    }

    let intersection = bigrams_a.intersection(&bigrams_b).count();
    let union = bigrams_a.union(&bigrams_b).count();

    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

/// Compute combined name similarity score (0.0 - 1.0)
fn name_similarity(name1: &str, name2: &str) -> f64 {
    let n1 = normalize_name(name1);
    let n2 = normalize_name(name2);

    if n1 == n2 { return 1.0; }

    let max_len = std::cmp::max(n1.len(), n2.len()).max(1);
    let lev_dist = levenshtein(&n1, &n2);
    let lev_sim = 1.0 - (lev_dist as f64 / max_len as f64);

    let jac_sim = jaccard_similarity(&n1, &n2);

    // Weighted average: Levenshtein 0.6, Jaccard 0.4
    lev_sim * 0.6 + jac_sim * 0.4
}

/// Find duplicate groups among installed skills
pub fn find_duplicates(skills: &[InstalledSkill]) -> Vec<(f64, String, Vec<InstalledSkill>)> {
    let mut groups: Vec<(f64, String, Vec<InstalledSkill>)> = Vec::new();
    let mut processed = vec![false; skills.len()];

    for i in 0..skills.len() {
        if processed[i] { continue; }

        let mut group = vec![skills[i].clone()];
        let mut max_similarity = 0.0_f64;

        for j in (i + 1)..skills.len() {
            if processed[j] { continue; }

            let sim = name_similarity(&skills[i].name, &skills[j].name);
            if sim > 0.6 {
                group.push(skills[j].clone());
                processed[j] = true;
                if sim > max_similarity { max_similarity = sim; }
            }
        }

        if group.len() > 1 {
            let level = if max_similarity > 0.8 { "high".to_string() } else { "medium".to_string() };
            groups.push((max_similarity, level, group));
        }
    }

    // Sort by similarity descending
    groups.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("hello", "hello"), 0);
        assert_eq!(levenshtein("hello", "hallo"), 1);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn test_jaccard() {
        assert!((jaccard_similarity("hello", "hello") - 1.0).abs() < 0.001);
        assert!(jaccard_similarity("abc", "def") < 0.1);
    }

    #[test]
    fn test_name_similarity() {
        let sim = name_similarity("pdf-parse", "pdf-extract");
        assert!(sim > 0.3);
        assert!(sim < 1.0);

        // Identical names should get 1.0
        assert!((name_similarity("code-review", "code-review") - 1.0).abs() < 0.001);
    }
}
