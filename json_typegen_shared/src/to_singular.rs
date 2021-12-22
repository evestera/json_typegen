const ENDS_WITH_RULES: &[(&str, usize, &str)] = &[
    ("series", 0, ""),
    ("cookies", 1, ""),
    ("movies", 1, ""),
    ("ies", 3, "y"),
    ("les", 1, ""),
    ("pes", 1, ""),
    ("ss", 0, ""),
    ("es", 0, ""),
    ("is", 0, ""),
    ("as", 0, ""),
    ("us", 0, ""),
    ("os", 0, ""),
    ("news", 0, ""),
    ("s", 1, ""),
];

/// Singularize a word for use as a type name
///
/// Implementation notes:
/// - Prefer to be conservative; Missing singularizations are better than incorrect ones.
/// - It's OK if this is somewhat use-case specific. It's not exposed.
/// - No regexes, since we don't want the regex dependency in the WASM.
pub fn to_singular(s: &str) -> String {
    let lowercase = s.to_ascii_lowercase();
    for (suffix, to_strip, replacement) in ENDS_WITH_RULES {
        if lowercase.ends_with(suffix) {
            return s[0..(s.len() - to_strip)].to_string() + replacement;
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_singular() {
        let mut incorrectly_singularized: Vec<(&'static str, &'static str, String)> = Vec::new();
        let mut not_singularized: Vec<(&'static str, &'static str)> = Vec::new();
        let mut assert_to_singular_matches = |input: &'static str, expected: &'static str| {
            let output = to_singular(input);
            if output != expected {
                if output == input {
                    not_singularized.push((input, expected))
                } else {
                    incorrectly_singularized.push((input, expected, output))
                }
            }
        };

        assert_to_singular_matches("cards", "card");
        assert_to_singular_matches("types", "type");
        assert_to_singular_matches("colors", "color");
        assert_to_singular_matches("rulings", "ruling");
        assert_to_singular_matches("foreignNames", "foreignName");
        assert_to_singular_matches("tags", "tag");
        assert_to_singular_matches("categoryKeys", "categoryKey");
        assert_to_singular_matches("attributes", "attribute");
        assert_to_singular_matches("values", "value");
        assert_to_singular_matches("images", "image");

        assert_to_singular_matches("guesses", "guess");

        assert_to_singular_matches("moves", "move");
        assert_to_singular_matches("lives", "life");
        assert_to_singular_matches("leaves", "leaf");

        assert_to_singular_matches("legalities", "legality");
        assert_to_singular_matches("abilities", "ability");
        assert_to_singular_matches("queries", "query");
        assert_to_singular_matches("cookies", "cookie");
        assert_to_singular_matches("movies", "movie");

        assert_to_singular_matches("matrices", "matrix");
        assert_to_singular_matches("vertices", "vertex");
        assert_to_singular_matches("indices", "index");
        assert_to_singular_matches("slices", "slice");

        assert_to_singular_matches("children", "child");

        assert_to_singular_matches("series", "series");
        assert_to_singular_matches("news", "news");
        assert_to_singular_matches("axis", "axis");

        if !not_singularized.is_empty() {
            println!(
                "Missed {} singularizations for to_singular() (input, expected):\n  {}\n\n",
                not_singularized.len(),
                not_singularized
                    .iter()
                    .map(|(input, expected)| format!("{}, {}", input, expected))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }

        if !incorrectly_singularized.is_empty() {
            panic!(
                "Test failures for to_singular() (input, expected, output):\n  {}\n\n",
                incorrectly_singularized
                    .iter()
                    .map(|(input, expected, output)| format!("{}, {}, {}", input, expected, output))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }
    }
}
