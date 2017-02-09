#[macro_use]
extern crate derive_swagger_2;

#[derive(Debug, Swagger)]
#[swagger(file = "swagger.json")]
struct Petstore;

fn main() {
    Petstore::foo();
}
