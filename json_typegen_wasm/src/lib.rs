use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn run(name: &str, input: &str, options: &str) -> String {
    set_panic_hook();

    let opts = match json_typegen_shared::parse::options(options) {
        Ok(opts) => opts,
        Err(msg) => return format!("Error: {}", msg),
    };

    match json_typegen_shared::codegen(name, input, opts) {
        Ok(res) => res,
        Err(err) => {
            let message = json_typegen_shared::internal_util::display_error_with_causes(&err);
            format!("Error: {}", message)
        }
    }
}
