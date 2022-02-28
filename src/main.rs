#[macro_use]
pub mod macros;

pub mod models;
pub mod config;
pub mod utils;

fn main() {
    let mut wm = models::WindowManager::init();
    wm.run();
}



