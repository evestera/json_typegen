use std::ascii::AsciiExt;

pub fn camel_case(name: &str) -> String {
    let mut s = String::new();
    let mut last = ' ';
    for c in name.chars() {
        if !c.is_alphabetic() {
            last = c;
            continue;
        }
        if !c.is_ascii() {
            last = c;
            continue;
        }
        if (!last.is_alphabetic() && c.is_alphabetic()) || (last.is_lowercase() && c.is_uppercase()) {
            s.push(c.to_ascii_uppercase());
        } else {
            s.push(c.to_ascii_lowercase());
        }
        last = c;
    }
    s
}

pub fn snake_case(name: &str) -> String {
    let mut s = String::new();
    let mut last = 'A';
    for c in name.chars() {
        if !c.is_alphabetic() {
            last = c;
            continue;
        }
        if !c.is_ascii() {
            last = c;
            continue;
        }
        if (!last.is_alphabetic() && c.is_alphabetic()) || (last.is_lowercase() && c.is_uppercase()) {
            s.push('_');
        }
        s.push(c.to_ascii_lowercase());
        last = c;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case() {
        assert_eq!("FooBar", &camel_case("FooBar"));
        assert_eq!("FooBar", &camel_case("fooBar"));
        assert_eq!("FooBar", &camel_case("foo bar"));
        assert_eq!("FooBar", &camel_case("foo_bar"));
        assert_eq!("FooBar", &camel_case("FOO_BAR"));
    }

    #[test]
    fn test_snake_case() {
        assert_eq!("foo_bar", &snake_case("FooBar"));
        assert_eq!("foo_bar", &snake_case("fooBar"));
        assert_eq!("foo_bar", &snake_case("foo bar"));
        assert_eq!("foo_bar", &snake_case("foo_bar"));
        assert_eq!("foo_bar", &snake_case("FOO_BAR"));
    }
}
