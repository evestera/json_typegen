extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rustfmt;
extern crate json_sample_shared;
extern crate error_chain;

use std::path::Path;

use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;
use rustfmt::config::{Config, WriteMode};
use error_chain::ChainedError;
use std::env;

use json_sample_shared::{codegen_from_sample, SampleSource};

#[derive(Debug, Deserialize)]
struct ReqBody {
    name: String,
    input: String
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
    let req_body: ReqBody = handle!(serde_json::de::from_reader(&mut req.body),
        "Error: Request body was invalid JSON");
    let tokens = handle!(codegen_from_sample(&req_body.name, &SampleSource::Text(&req_body.input)),
        err => format!("{}", err.display()));

    let input = rustfmt::Input::Text(String::from(tokens.as_str()));
    let mut output: Vec<u8> = Vec::new();
    let mut config = Config::default();
    config.write_mode = WriteMode::Plain;
    handle!(rustfmt::format_input(input, &config, Some(&mut output)),
        err => format!("Error formatting output with rustfmt: {:?}", err));

    let s = String::from_utf8_lossy(&output);
    let formatted = fix_rustfmt_issues(&s);

    Ok(Response::with((status::Ok, formatted)))
}

fn fix_rustfmt_issues(input: &str) -> String {
    let mut output = String::new();

    for line in input.lines() {
        if line.starts_with('#') || line.starts_with("    #") {
            if line.starts_with("    #") {
                output.push_str("    ");
            }
            for c in line.chars() {
                match c {
                    ' ' => {},
                    ',' => { output.push_str(", "); }
                    _ => { output.push(c); }
                }
            }
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
            if line == "}" {
                output.push('\n');
            }
        }
    }

    output
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
