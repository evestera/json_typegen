// Words (especially ending in s) that are already singular and should be left as is
const ALREADY_SINGULAR: &[&str] = &[
    "series",
    "species",
    "news",
    "clothes",
];

// (ends_with, strip_chars, append)
const ENDS_WITH_RULES: &[(&str, usize, &str)] = &[
    ("zes", 3, ""),
    ("matrices", 4, "ix"),
    ("appendices", 4, "ix"),
    ("radices", 4, "ix"),
    ("vertices", 4, "ex"),
    ("vortices", 4, "ex"),
    ("indices", 4, "ex"),
    ("codices", 4, "ex"),
    ("people", 5, "erson"),
    ("cookies", 1, ""),
    ("movies", 1, ""),
    ("ives", 4, "ife"),
    ("lves", 4, "lf"),
    ("rves", 4, "rf"),
    ("buses", 2, ""),
    ("gasses", 3, ""),
    ("children", 3, ""),
    ("feet", 3, "oot"),
    ("leaves", 3, "f"),
    ("staves", 3, "ff"),
    ("thieves", 3, "f"),
    ("fishes", 2, ""),
    ("taxies", 2, ""),
    ("ies", 3, "y"),
    ("oes", 2, ""),
    ("les", 1, ""),
    ("pes", 1, ""),
    ("ss", 0, ""),
    ("sses", 2, ""),
    ("yses", 2, "is"),
    ("diagnoses", 2, "is"),
    ("prognoses", 2, "is"),
    ("synopses", 2, "is"),
    ("crises", 2, "is"),
    ("theses", 2, "is"),
    ("statuses", 2, ""),
    ("aliases", 2, ""),
    ("ses", 1, ""),
    ("xes", 2, ""),
    ("schemas", 1, ""),
    ("as", 0, ""),
    ("es", 1, ""),
    ("taxis", 1, ""),
    ("is", 0, ""),
    ("os", 0, ""),
    ("menus", 1, ""),
    ("us", 0, ""),
    ("s", 1, ""),
];

/// Singularize a word for use as a type name
///
/// Implementation notes:
/// - Prefer to be conservative; Missing singularizations are better than incorrect ones.
/// - It's OK if this is somewhat use-case specific. It's not exposed.
/// - No regexes, since we don't want the regex dependency in the WASM.
/// - We generally want to avoid replacing entire words, so that we don't need complex logic for preserving case
///
/// Known issues:
/// -
pub fn to_singular(s: &str) -> String {
    let lowercase = s.to_ascii_lowercase();
    for suffix in ALREADY_SINGULAR {
        if lowercase.ends_with(suffix) {
            return s.to_string()
        }
    }
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
    fn test_to_singular_from_file() {
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

        let plurals_text = include_str!("to_singular_tests.txt");
        // let plurals_text = include_str!("plurals.txt");
        for line in plurals_text.lines() {
            if line.trim().is_empty() { continue }
            let split = line.split(' ').collect::<Vec<_>>();
            let plural = split[0];
            let singular = split[1];
            assert_to_singular_matches(plural, singular);
        }

        if !not_singularized.is_empty() || !incorrectly_singularized.is_empty() {
            println!("Incorrect: {}, Missed: {}\n", incorrectly_singularized.len(), not_singularized.len());
        }

        if !not_singularized.is_empty() {
            println!(
                "Missed {} singularizations for to_singular() (input, expected):\n  {}\n",
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
                "Test failures ({}) for to_singular() (input, expected, output):\n  {}\n",
                incorrectly_singularized.len(),
                incorrectly_singularized
                    .iter()
                    .map(|(input, expected, output)| format!("{}, {}, {}", input, expected, output))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }
    }
}
