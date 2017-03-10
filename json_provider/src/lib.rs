#[macro_use]
extern crate json_sample_derive;
#[macro_use]
extern crate serde_derive;

pub use json_sample_derive::*;
pub use serde_derive::*;

#[macro_export]
macro_rules! json_provider {
    ($name:expr, $source:expr) => {
        #[derive(json_sample)]
        #[json_sample(name = $name)]
        #[json_sample(source = $source)]
        #[allow(unused)]
        struct JsonProviderPlaceholder;
    }
}
