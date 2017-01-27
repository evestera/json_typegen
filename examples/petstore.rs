#![feature(proc_macro)]

#[macro_use]
extern crate derive_swagger_2;

#[derive(Debug, Swagger)]
#[url = "http://petstore.swagger.io/v2/swagger.json"]
struct Petstore;

fn main() {
    Petstore::foo();
}
