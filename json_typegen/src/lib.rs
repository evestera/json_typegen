#[allow(unused_imports)]
#[macro_use]
extern crate json_typegen_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

pub use json_typegen_derive::*;
pub use serde_derive::*;

#[macro_export]
macro_rules! json_typegen {
    ($name:expr, $source:expr) => {
        #[derive(json_types)]
        #[json_typegen(name = $name)]
        #[json_typegen(source = $source)]
        #[allow(unused)]
        struct JsonProviderPlaceholder;
    }
}
