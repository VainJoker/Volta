use xcb::x::KeyButMask;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct KeyCode {
    pub mask: KeyButMask,
    pub code: u8,
}

impl KeyCode {
    pub fn from_key_press(k: &xcb::x::KeyPressEvent) -> KeyCode {
        KeyCode {
            mask: k.state(),
            code: k.detail(),
        }
    }
}
