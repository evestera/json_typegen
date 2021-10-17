// conservative placeholder impl for now to pass the tests
pub fn to_singular(s: &str) -> String {
    if s.ends_with("les") || s.ends_with("pes") {
        return s[0..s.len() - 1].to_string();
    } else if s.ends_with("ss")
        || s.ends_with("es")
        || s.ends_with("is")
        || s.ends_with("as")
        || s.ends_with("us")
        || s.ends_with("os")
        || s.ends_with("news")
    {
        return s.to_string();
    } else if s.ends_with('s') {
        return s[0..s.len() - 1].to_string();
    }
    s.to_string()
}
