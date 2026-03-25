use rocket_include_dir::{Dir, StaticFiles, include_dir};

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    static STATIC_DIR: Dir = include_dir!("static");
    rocket::build().mount("/", StaticFiles::from(&STATIC_DIR))
}
