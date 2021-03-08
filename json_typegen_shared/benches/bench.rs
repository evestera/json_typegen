#![cfg_attr(feature = "unstable", feature(test))]
#![cfg(all(feature = "unstable", test))]

// Benchmark currently only works on nightly Rust due to extern crate test
// Tracking issue: https://github.com/rust-lang/rust/issues/29553
// Running:
// cargo +nightly bench --features unstable

extern crate json_typegen_shared;
extern crate test;

use json_typegen_shared::{codegen, Options};
use test::Bencher;

macro_rules! file_bench {
    ($name:ident, $file_path:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| codegen("Article", include_str!($file_path), Options::default()));
        }
    };
}

file_bench!(magic_card_list, "fixtures/magic_card_list.json");
file_bench!(zalando_article, "fixtures/zalando_article.json");
