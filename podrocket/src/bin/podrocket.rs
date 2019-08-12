#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{get, routes};

#[get("/")]
fn hello() -> &'static str {
    "Hello world"
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
