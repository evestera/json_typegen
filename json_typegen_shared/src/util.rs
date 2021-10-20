pub fn camel_case(name: &str) -> String {
    let mut s = String::new();
    let mut last = ' ';
    for c in name.chars().skip_while(|c| !c.is_ascii_alphanumeric()) {
        if !c.is_ascii_alphanumeric() {
            last = c;
            continue;
        }
        if (last.is_ascii() && !last.is_ascii_alphanumeric() && c.is_ascii_alphanumeric())
            || (last.is_ascii_lowercase() && c.is_ascii_uppercase())
        {
            s.push(c.to_ascii_uppercase());
        } else if last.is_ascii_alphabetic() {
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
        last = c;
    }
    s
}

pub fn snake_case(name: &str) -> String {
    sep_case(name, '_')
}

pub fn kebab_case(name: &str) -> String {
    sep_case(name, '-')
}

fn sep_case(name: &str, separator: char) -> String {
    let mut s = String::new();
    let mut last = 'A';
    for c in name.chars().skip_while(|c| !c.is_ascii_alphanumeric()) {
        if !c.is_ascii_alphanumeric() {
            last = c;
            continue;
        }
        if (last.is_ascii() && !last.is_ascii_alphanumeric() && c.is_ascii_alphanumeric())
            || (last.is_ascii_lowercase() && c.is_ascii_uppercase())
        {
            s.push(separator);
        }
        s.push(c.to_ascii_lowercase());
        last = c;
    }
    s
}

pub fn type_case(name: &str) -> String {
    let s = camel_case(name);
    uppercase_first_letter(&s)
}

pub fn lower_camel_case(name: &str) -> String {
    let s = camel_case(name);
    lowercase_first_letter(&s)
}

// from http://stackoverflow.com/questions/38406793/.../38406885
fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
    }
}

fn lowercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_ascii_lowercase().to_string() + c.as_str(),
    }
}

// based on hashmap! macro from maplit crate
macro_rules! string_hashmap {
    ($($key:expr => $value:expr,)+) => { string_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut _map = ::linked_hash_map::LinkedHashMap::new();
            $(
                _map.insert($key.to_string(), $value);
            )*
            _map
        }
    };
}

pub(crate) use string_hashmap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case() {
        assert_eq!("FooBar", &camel_case("FooBar"));
        assert_eq!("FooBar", &camel_case("fooBar"));
        assert_eq!("FooBar", &camel_case("foo bar"));
        assert_eq!("FooBar", &camel_case("foo_bar"));
        assert_eq!("FooBar", &camel_case("_foo_bar"));
        assert_eq!("FooBar", &camel_case("åfoo_bar"));
        assert_eq!("FooBar", &camel_case("foåo_bar"));
        assert_eq!("FooBar", &camel_case("FOO_BAR"));

        assert_eq!("Foo1bar", &camel_case("Foo1bar"));
        assert_eq!("Foo2bar", &camel_case("foo_2bar"));
        assert_eq!("Foo3Bar", &camel_case("Foo3Bar"));
        assert_eq!("Foo4Bar", &camel_case("foo4_bar"));
        assert_eq!("1920x1080", &camel_case("1920x1080"));
        assert_eq!("19201080", &camel_case("1920*1080"));
    }

    #[test]
    fn test_snake_case() {
        assert_eq!("foo_bar", &snake_case("FooBar"));
        assert_eq!("foo_bar", &snake_case("fooBar"));
        assert_eq!("foo_bar", &snake_case("foo bar"));
        assert_eq!("foo_bar", &snake_case("foo_bar"));
        assert_eq!("foo_bar", &snake_case("_foo_bar"));
        assert_eq!("foo_bar", &snake_case("åfoo_bar"));
        assert_eq!("foo_bar", &snake_case("foåo_bar"));
        assert_eq!("foo_bar", &snake_case("FOO_BAR"));

        assert_eq!("foo_5bar", &snake_case("foo_5bar"));
        assert_eq!("foo6_bar", &snake_case("foo6_bar"));
        assert_eq!("1920x1080", &snake_case("1920x1080"));
        assert_eq!("1920_1080", &snake_case("1920*1080"));
    }
}
