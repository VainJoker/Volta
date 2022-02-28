use std::process;
use xcb::x::KeyButMask;
use crate::models::{CodeMap,KeyCode};
pub fn keycodes_from_xmodmap() -> CodeMap {
    match process::Command::new("xmodmap").arg("-pke").output() {
        Err(e) => die!("unable to fetch keycodes via xmodmap: {}", e),
        Ok(o) => match String::from_utf8(o.stdout) {
            Err(e) => die!("invalid utf8 from xmodmap: {}", e),
            Ok(s) => s
                .lines()
                .flat_map(|l| {
                    let mut words = l.split_whitespace(); // keycode <code> = <names ...>
                    let key_code: u8 = words.nth(1).unwrap().parse().unwrap();
                    words.skip(1).map(move |name| (name.into(), key_code))
                })
                .collect::<CodeMap>(),
        },
    }
}
pub fn parse_key_binding<S>(pattern: S, known_codes: &CodeMap) -> Option<KeyCode>
where
    S: Into<String>,
{
    let s = pattern.into();
    let mut parts: Vec<&str> = s.split("-").collect();
    match known_codes.get(parts.remove(parts.len() - 1)) {
        Some(code) => {
            let mask : KeyButMask = parts
                .iter()
                .map(|s| match s {
                    &"A" => xcb::x::KeyButMask::MOD1,
                    &"M" => xcb::x::KeyButMask::MOD4,
                    &"S" => xcb::x::KeyButMask::SHIFT,
                    &"C" => xcb::x::KeyButMask::CONTROL,
                    &_ => die!("invalid key binding prefix: {}", s),
                })
                .fold(KeyButMask::empty(), |acc, v| acc | v);

            // log!("binding '{}' as [{}, {}]", s, mask, code);
            Some(KeyCode {
                mask,
                code: *code,
            })
        }
        None => None,
    }
}
