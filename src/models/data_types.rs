use std::collections::HashMap;
use crate::models::wm::WindowManager;
use crate::models::key::KeyCode;
// pub type LayoutFunc = Box<dyn Fn(usize, &Region, usize, f32) -> Vec<Region>>;
pub type FireAndForget = Box<dyn Fn(&mut WindowManager) -> ()>;
pub type KeyBindings = HashMap<KeyCode, FireAndForget>;
pub type CodeMap = HashMap<String, u8>;
