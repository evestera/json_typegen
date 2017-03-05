#[macro_use]
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rustfmt;
extern crate json_sample_shared;

use std::path::Path;

use iron::prelude::*;
use iron::status;
use staticfile::Static;
use mount::Mount;
use rustfmt::config::{Config, WriteMode};

use json_sample_shared::{codegen_from_sample, SampleSource};

#[derive(Debug, Deserialize)]
struct ReqBody {
    name: String,
    input: String
}

fn hello_world(req: &mut Request) -> IronResult<Response> {
    let req_body: ReqBody = itry!(serde_json::de::from_reader(&mut req.body),
        (status::BadRequest, "Invalid JSON"));
    let tokens = match codegen_from_sample(&req_body.name, SampleSource::Text(&req_body.input)) {
        Ok(tokens) => tokens,
        Err(_) => return Ok(Response::with((status::BadRequest, "Unable to generate code")))
    };

    let input = rustfmt::Input::Text(String::from(tokens.as_str()));
    let mut output: Vec<u8> = Vec::new();
    let mut config = Config::default();
    config.write_mode = WriteMode::Plain;
    rustfmt::format_input(input, &config, Some(&mut output)).unwrap();

    Ok(Response::with((status::Ok, output)))
}

fn main() {
    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("static/")));
    mount.mount("/api", hello_world);

    let host = "localhost:3000";
    let _server = Iron::new(mount).http(host).unwrap();
    println!("Serving on {}", host);
}
