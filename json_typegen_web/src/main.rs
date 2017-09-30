extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate json_typegen_shared;
extern crate error_chain;

use std::path::Path;

use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;
use error_chain::ChainedError;
use std::env;

use json_typegen_shared::{codegen, Options};

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ReqBody {
    name: String,
    input: String,
    runnable: bool,
    derives: String,
}

macro_rules! handle {
    ($result:expr, $err_handler:expr) => (handle!($result, _err => $err_handler));
    ($result:expr, $err_id:ident => $err_handler:expr) => (match $result {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err($err_id) => return ::std::result::Result::Ok(
            ::iron::Response::with((::iron::status::BadRequest, $err_handler)))
    });
}

fn handle_codegen_request(req: &mut Request) -> IronResult<Response> {
    let mut req_body: ReqBody = handle!(serde_json::de::from_reader(&mut req.body),
        "Error: Request body was invalid JSON");
    let mut options = Options::default();
    options.runnable = req_body.runnable;
    if !(req_body.derives.trim().is_empty()) {
        options.derives = req_body.derives;
    }
    if req_body.input.trim().is_empty() {
        req_body.input.push_str("{}");
    }
    let code = handle!(codegen(&req_body.name, &req_body.input, options),
        err => format!("{}", err.display()));

    Ok(Response::with((status::Ok, code)))
}

fn main() {
    let mut mount = Mount::new();

    let p = env::current_dir().unwrap();
    println!("Working directory is {}", p.display());
    let static_path = Path::new("static/");
    let canonical_static = static_path.canonicalize().unwrap();
    println!("Looking for static files in {}", canonical_static.display());

    mount.mount("/", Static::new(static_path));
    mount.mount("/api", handle_codegen_request);

    let host = env::var("JSONSAMPLE_HOST").unwrap_or_else(|_| String::from("0.0.0.0:5555"));
    let _server = Iron::new(mount).http(&host as &str).unwrap();
    println!("Serving on http://{}", host);
}
