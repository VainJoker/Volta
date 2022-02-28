#[macro_use]
pub mod helper;

pub mod config;
pub mod models;

fn main() {
    let mut wm = models::wm::WindowManager::init();
    wm.run();
}
