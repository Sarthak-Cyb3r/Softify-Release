use std::collections::HashSet;

/// Unicode whitespace characters that must be normalized to standard ASCII space ' '.
/// Includes \u{00A0} (NBSP), en quad, em quad, en space, em space, three-per-em space,
/// four-per-em space, six-per-em space, figure space, punctuation space, thin space,
/// hair space, zero-width space, narrow no-break space, and ideographic space.
const UNICODE_SPACES: &[char] = &[
    '\u{00A0}', // Non-breaking space
    '\u{2000}', // En quad
    '\u{2001}', // Em quad
    '\u{2002}', // En space
    '\u{2003}', // Em space
    '\u{2004}', // Three-per-em space
    '\u{2005}', // Four-per-em space
    '\u{2006}', // Six-per-em space
    '\u{2007}', // Figure space
    '\u{2008}', // Punctuation space
    '\u{2009}', // Thin space
    '\u{200A}', // Hair space
    '\u{200B}', // Zero-width space
    '\u{202F}', // Narrow no-break space
    '\u{3000}', // Ideographic space
];

pub const FORBIDDEN_VARIANTS: &[&str] = &[
    "remix", "mashup", "dj", "lofi", "lo-fi", "slowed", "reverb", "acoustic", "cover", "karaoke",
];

/// Normalizes Unicode whitespace and punctuation according to Softify specifications.
/// Mandatory: Replaces \u{00A0} and all Unicode whitespace with standard ASCII space ' '.
pub fn normalize_unicode_spaces(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        if UNICODE_SPACES.contains(&ch) {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    // Collapse consecutive spaces into a single space and trim
    let collapsed: Vec<&str> = result.split_whitespace().collect();
    collapsed.join(" ")
}

/// Normalizes text for comparison: lowercases, normalizes whitespace, standardizes quotes.
pub fn normalize_canonical(input: &str) -> String {
    let space_normalized = normalize_unicode_spaces(input);
    let mut normalized = String::with_capacity(space_normalized.len());

    for ch in space_normalized.chars() {
        match ch {
            '’' | '‘' | '`' => normalized.push('\''),
            '“' | '”' => normalized.push('"'),
            _ => normalized.push(ch),
        }
    }

    normalized.to_lowercase()
}

/// Checks if a string contains any variant terms (e.g. remix, acoustic, cover, etc.)
pub fn contains_variant(text: &str) -> bool {
    let lower = text.to_lowercase();
    for variant in FORBIDDEN_VARIANTS {
        if lower.contains(variant) {
            return true;
        }
    }
    false
}

/// Determines whether a candidate should be disqualified due to variant terms,
/// unless the original user query explicitly asked for that variant.
pub fn is_disqualified_variant(candidate: &str, user_query: &str) -> bool {
    let query_lower = user_query.to_lowercase();
    let candidate_lower = candidate.to_lowercase();

    for variant in FORBIDDEN_VARIANTS {
        if candidate_lower.contains(variant) && !query_lower.contains(variant) {
            return true;
        }
    }
    false
}

/// Evaluates duration proximity between candidate and target duration.
/// - If discrepancy > 90 seconds, candidate is rejected (returns None).
/// - If discrepancy <= 6 seconds, returns a bonus of +40 points.
/// - Otherwise, returns a base score scaled down by the delta.
pub fn score_duration_proximity(candidate_seconds: u32, target_seconds: u32) -> Option<i32> {
    let delta = (candidate_seconds as i32 - target_seconds as i32).abs();
    if delta > 90 {
        return None; // Reject candidate: duration discrepancy > 90s
    }

    if delta <= 6 {
        Some(40) // +40 point bonus for accurate match
    } else {
        Some((90 - delta) * 20 / 90)
    }
}

/// Cleans title by removing common video/audio suffixes (e.g. "(Official Video)", "[Lyrics]").
pub fn clean_title(title: &str) -> String {
    let mut cleaned = normalize_unicode_spaces(title);

    // List of suffixes to strip
    let patterns = [
        "(official video)",
        "[official video]",
        "(official music video)",
        "[official music video]",
        "(official audio)",
        "[official audio]",
        "(lyrics)",
        "[lyrics]",
        "(audio)",
        "[audio]",
        "(visualizer)",
        "[visualizer]",
    ];

    for pattern in patterns {
        if let Some(pos) = cleaned.to_lowercase().find(pattern) {
            cleaned.replace_range(pos..pos + pattern.len(), "");
        }
    }

    normalize_unicode_spaces(&cleaned)
}

/// Computes token set similarity between two text strings (Jaccard-like ratio).
pub fn token_similarity(a: &str, b: &str) -> f32 {
    let norm_a = normalize_canonical(a);
    let norm_b = normalize_canonical(b);

    let tokens_a: HashSet<&str> = norm_a
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    let tokens_b: HashSet<&str> = norm_b
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let intersection_count = tokens_a.intersection(&tokens_b).count();
    let union_count = tokens_a.union(&tokens_b).count();

    if union_count == 0 {
        0.0
    } else {
        intersection_count as f32 / union_count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_whitespace_normalization() {
        // Spotify multi-artist separator: ",\u{00A0}"
        let input = "The Weeknd,\u{00A0}Daft Punk";
        let normalized = normalize_unicode_spaces(input);
        assert_eq!(normalized, "The Weeknd, Daft Punk");

        // Em space, thin space, zero-width space
        let mixed = "Artist\u{2003}Name\u{2009}Feat\u{200B}Guest";
        let norm_mixed = normalize_unicode_spaces(mixed);
        assert_eq!(norm_mixed, "Artist Name Feat Guest");
    }

    #[test]
    fn test_variant_disqualification() {
        assert!(is_disqualified_variant("Starboy (Remix)", "Starboy"));
        assert!(is_disqualified_variant("Blinding Lights (Acoustic)", "Blinding Lights"));
        assert!(is_disqualified_variant("Save Your Tears [Lofi Cover]", "Save Your Tears"));

        // Allowed if user explicitly asked for remix
        assert!(!is_disqualified_variant("Starboy (Remix)", "Starboy Remix"));
        assert!(!is_disqualified_variant("Shape of You Acoustic", "Shape of You Acoustic"));
    }

    #[test]
    fn test_duration_proximity_scoring() {
        // Exact or close match (within 6s): +40 points
        assert_eq!(score_duration_proximity(200, 204), Some(40));
        assert_eq!(score_duration_proximity(200, 206), Some(40));

        // Acceptable match (delta <= 90s)
        let intermediate = score_duration_proximity(200, 240);
        assert!(intermediate.is_some());
        assert!(intermediate.unwrap() < 40);

        // Reject match (> 90s discrepancy)
        assert_eq!(score_duration_proximity(200, 300), None);
    }

    #[test]
    fn test_clean_title() {
        let title = "Blinding Lights (Official Music Video)";
        assert_eq!(clean_title(title), "Blinding Lights");

        let audio_title = "Starboy [Official Audio]";
        assert_eq!(clean_title(audio_title), "Starboy");
    }

    #[test]
    fn test_token_similarity() {
        let score = token_similarity("The Weeknd - Blinding Lights", "Blinding Lights The Weeknd");
        assert!(score > 0.8);
    }
}
