#[macro_use]
extern crate swagger_derive;

#[derive(Debug, Swagger)]
#[swagger(file = "../petstore.json")]
struct Petstore;

fn main() {
    Petstore::foo();
}
