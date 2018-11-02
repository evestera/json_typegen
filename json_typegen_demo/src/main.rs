#[macro_use]
extern crate json_typegen;
use serde_json;

json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);

fn main() {
    let mut p: Point = serde_json::from_str(r#"{ "x": 3, "y": 5 }"#).unwrap();
    println!("deserialized = {:?}", p);
    p.x = 4;
    let serialized = serde_json::to_string(&p).unwrap();
    println!("serialized = {}", serialized);
}
