#![allow(dead_code)]

use std::borrow::Cow;
use std::cell::Cell;

#[derive(Debug, PartialEq, Clone)]
pub enum HintType {
    OpaqueType(String),
    MapType(String),
    TypeName(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Hint {
    pub hint_type: HintType,
    pub used: Cell<bool>,
}

impl Hint {
    pub fn new(hint_type: HintType) -> Self {
        Hint {
            hint_type,
            used: Cell::new(false),
        }
    }

    pub fn default_map() -> Self {
        Hint::new(HintType::MapType("::std::collections::HashMap".into()))
    }

    pub fn opaque_type(name: impl Into<String>) -> Self {
        Hint::new(HintType::OpaqueType(name.into()))
    }

    pub fn type_name(name: impl Into<String>) -> Self {
        Hint::new(HintType::TypeName(name.into()))
    }
}

pub struct Hints<'a> {
    pub hints: Vec<(Cow<'a, [&'a str]>, &'a Hint)>,
    pub applicable: Vec<&'a Hint>,
}

fn is_index(s: &str) -> bool {
    s == "-" || s.bytes().all(|b| (b'0'..=b'9').contains(&b))
}

impl<'a> Hints<'a> {
    pub fn new() -> Self {
        Hints {
            hints: Vec::new(),
            applicable: Vec::new(),
        }
    }

    pub fn add(&mut self, pointer: &'a str, hint: &'a Hint) {
        if pointer.is_empty() {
            self.applicable.push(hint);
        } else {
            if !pointer.starts_with('/') {
                panic!(
                    "Invalid JSON pointer: {:?}\n{}",
                    pointer, "A pointer not referring to the root has to start with '/'",
                );
            }
            let tokens: Vec<_> = pointer.split('/').skip(1).collect();
            let pair: (Cow<[&str]>, &Hint) = (tokens.into(), hint);
            self.hints.push(pair);
        }
    }

    /// ([/a/b, /a/c, /d/e], "a") -> [/b, /c]
    pub fn step_field(&self, name: &str) -> Hints {
        self.step(|first| first == "-" || first == name)
    }

    /// [/1/b, /a/c, /-/e] -> [/b, /c, /e]
    pub fn step_any(&self) -> Hints {
        self.step(|_first| true)
    }

    /// [/1/b, /a/c, /-/e] -> [/b, /e]
    pub fn step_array(&self) -> Hints {
        self.step(is_index)
    }

    /// ([/2/b, /a/c, /-/e, /3/d], 3) -> [/e, /d]
    pub fn step_index(&self, index: usize) -> Hints {
        let i_str = &index.to_string();
        self.step(|first| first == "-" || first == i_str)
    }

    fn step<F: Fn(&str) -> bool>(&self, pred: F) -> Hints {
        let mut filtered = Vec::new();
        let mut applicable = Vec::new();

        for &(ref path, hint) in self.hints.iter() {
            if pred(path[0]) {
                let stepped: &[&str] = &path[1..];
                if stepped.is_empty() {
                    applicable.push(hint);
                } else {
                    filtered.push((stepped.into(), hint));
                }
            }
        }

        Hints {
            hints: filtered,
            applicable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pointers() {
        let hint = Hint::default_map();
        let mut hints = Hints::new();
        hints.add("/a/b", &hint);
        hints.add("/foo", &hint);
        hints.add("/foo", &hint);
        hints.add("", &hint);

        assert_eq!(hints.hints.len(), 3);
        assert_eq!(hints.applicable.len(), 1);
    }

    #[test]
    #[should_panic]
    fn invalid_pointer() {
        let hint = Hint::default_map();
        let mut hints = Hints::new();
        hints.add("foo", &hint);
    }

    #[test]
    fn step_field() {
        let hint = Hint::default_map();
        let mut hints = Hints::new();
        hints.add("/a/b", &hint);
        hints.add("/b/c", &hint);
        hints.add("/b/", &hint);
        hints.add("/b", &hint);

        let hints = hints.step_field("b");

        assert_eq!(hints.hints.len(), 2);
        assert_eq!(hints.applicable.len(), 1);
    }
}
