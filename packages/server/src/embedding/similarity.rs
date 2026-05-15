/// Compute TF-IDF cosine similarity matrix for a list of descriptions.
///
/// Returns an N x N matrix where `result[i][j]` is the cosine similarity
/// between description i and description j. Diagonal entries are 1.0.
///
/// This is a pure Rust implementation with no external ML dependencies,
/// suitable for MVP use.
pub fn compute_similarities(descriptions: Vec<String>) -> Vec<Vec<f64>> {
    let n = descriptions.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![vec![1.0]];
    }

    // Tokenize each description
    let tokenized: Vec<Vec<String>> = descriptions
        .iter()
        .map(|d| tokenize(d))
        .collect();

    // Build vocabulary: all unique terms across all descriptions
    let mut vocabulary: Vec<String> = Vec::new();
    let mut term_set: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for tokens in &tokenized {
        for token in tokens {
            if term_set.insert(token) {
                vocabulary.push(token.clone());
            }
        }
    }

    let vocab_size = vocabulary.len();

    // Compute TF (term frequency) matrices
    // TF[i][j] = frequency of term j in document i
    let mut tf = vec![vec![0.0_f64; vocab_size]; n];
    for (doc_idx, tokens) in tokenized.iter().enumerate() {
        let total_terms = tokens.len() as f64;
        if total_terms == 0.0 {
            continue;
        }
        for (term_idx, term) in vocabulary.iter().enumerate() {
            let count = tokens.iter().filter(|t| *t == term).count() as f64;
            tf[doc_idx][term_idx] = count / total_terms;
        }
    }

    // Compute IDF (inverse document frequency)
    // IDF[j] = log(N / df_j) where df_j is the number of documents containing term j
    let mut idf = vec![0.0_f64; vocab_size];
    for (term_idx, term) in vocabulary.iter().enumerate() {
        let doc_freq = tokenized
            .iter()
            .filter(|tokens| tokens.contains(term))
            .count() as f64;
        if doc_freq > 0.0 {
            idf[term_idx] = (n as f64 / doc_freq).ln();
        }
    }

    // Compute TF-IDF vectors
    let tfidf: Vec<Vec<f64>> = tf
        .iter()
        .map(|tf_vec| {
            tf_vec
                .iter()
                .zip(idf.iter())
                .map(|(t, i)| t * i)
                .collect()
        })
        .collect();

    // Compute cosine similarity matrix
    let mut similarities = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                similarities[i][j] = 1.0;
            } else if j > i {
                let sim = cosine_similarity(&tfidf[i], &tfidf[j]);
                similarities[i][j] = sim;
                similarities[j][i] = sim;
            }
        }
    }

    similarities
}

/// Tokenize a text string into lowercase alphanumeric tokens.
/// Filters out tokens shorter than 3 characters.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() >= 3)
        .map(|s| s.to_string())
        .collect()
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = compute_similarities(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_description() {
        let result = compute_similarities(vec!["hello world".to_string()]);
        assert_eq!(result.len(), 1);
        assert!((result[0][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_identical_descriptions() {
        let result = compute_similarities(vec!["hello world".to_string(), "hello world".to_string()]);
        assert_eq!(result.len(), 2);
        assert!((result[0][1] - 1.0).abs() < 1e-10);
        assert!((result[1][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_different_descriptions() {
        let result = compute_similarities(vec![
            "python programming language".to_string(),
            "rust systems programming".to_string(),
        ]);
        assert_eq!(result.len(), 2);
        // They share "programming" so similarity should be > 0 but < 1
        assert!(result[0][1] > 0.0);
        assert!(result[0][1] < 1.0);
    }

    #[test]
    fn test_tokenization() {
        let tokens = tokenize("Hello, World! This is a TEST.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"this".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        // "is" and "a" are too short
        assert!(!tokens.contains(&"is".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
    }
}
