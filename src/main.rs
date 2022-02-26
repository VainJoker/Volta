use xcb::randr::NotifyMask;
use std::process;
use std::collections::HashMap;
use xcb::{self, x::{self,KeyButMask}};

fn main() {
    let mut wm = WindowManager::init();
    wm.run();
}

/// log the reason why we we're dying and run cleanup (if any)
#[macro_export]
macro_rules! die(
    ($msg:expr) => ({
        eprintln!("FATAL :: {}", $msg);
        ::std::process::exit(42);
     });

    ($fmt:expr, $($arg:expr),*) => ({
        eprintln!("FATAL :: {}", format!($fmt, $($arg,)*));
        ::std::process::exit(42);
     });
);

#[macro_export]
macro_rules! warn(
    ($msg:expr) => { eprintln!("WARN :: {}", $msg); };
    ($fmt:expr, $($arg:tt),*) => {
        eprintln!("WARN :: {}", format!($fmt, $($arg)*))
    };
);

#[macro_export]
macro_rules! log(
    ($msg:expr) => { eprintln!("INFO :: {}", $msg); };
    ($fmt:expr, $($arg:expr),*) => {
        eprintln!("INFO :: {}", format!($fmt, $($arg,)*));
    };
);

/// kick off an external program as part of a key/mouse binding
#[macro_export]
macro_rules! run_external(
    ($cmd:tt) => {
        {
            let parts: Vec<&str> = $cmd.split_whitespace().collect();
            if parts.len() > 1 {
                Box::new(move |_: &mut WindowManager| {
                    match ::std::process::Command::new(parts[0]).args(&parts[1..]).spawn() {
                        Ok(_) => (),
                        Err(e) => warn!("error spawning external program: {}", e),
                    };
                }) as FireAndForget
            } else {
                Box::new(move |_: &mut WindowManager| {
                    match ::std::process::Command::new(parts[0]).spawn() {
                        Ok(_) => (),
                        Err(e) => warn!("error spawning external program: {}", e),
                    };
                }) as FireAndForget
            }
        }
    };
);

/// kick off an internal method on the window manager as part of a key/mouse binding
#[macro_export]
macro_rules! run_internal(
    ($func:ident) => {
        Box::new(|wm: &mut WindowManager| {
            log!("calling method ({})", stringify!($func));
            wm.$func()
        })
    };

    ($func:ident, $arg:tt) => {
        Box::new(move |wm: &mut WindowManager| wm.$func($arg))
    };
);

/// make creating a hash-map a little less verbose
#[macro_export]
macro_rules! map(
    {} => { ::std::collections::HashMap::new(); };

    { $($key:expr => $value:expr),+, } => {
        {
            let mut _map = ::std::collections::HashMap::new();
            $(_map.insert($key, $value);)+
            _map
        }
    };
);

/// make creating a hash-map a little less verbose
#[macro_export]
macro_rules! gen_keybindings(
    {
        $($binding:expr => $action:expr),+;
        // forall_tags: $tag_array:expr => { $($tag_binding:expr => $tag_action:tt),+, }
    } => {
        {
            let mut _map = ::std::collections::HashMap::new();
            let keycodes = keycodes_from_xmodmap();

            $(
                match parse_key_binding($binding, &keycodes) {
                    Some(key_code) => _map.insert(key_code, $action),
                    None => die!("invalid key binding: {}", $binding),
                };
            )+

            _map
        }
    };
);
pub struct WindowManager {
    conn: xcb::Connection,
    _screen_num: i32,
}

impl WindowManager {
    pub fn init() -> WindowManager {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

        let mut wm = WindowManager {
            conn,
            _screen_num: screen_num,
        };

        wm.update_screen_dimensions();
        wm
    }

    fn grab_keys(&self, key_bindings: &KeyBindings) {
        let screen = self.conn.get_setup().roots().nth(0).unwrap();
        let root = screen.root();
        // xcb docs: https://www.mankier.com/3/xcb_randr_select_input
        let input = xcb::randr::SelectInput{window:root, enable:NotifyMask::CRTC_CHANGE};
        self.conn.send_request(&input);
        for k in key_bindings.keys() {
            // xcb docs: https://www.mankier.com/3/xcb_grab_key
            xcb::x::GrabKey{
                owner_events: false,
                grab_window: root,
                modifiers: x::ModMask::ANY,
                key: k.code,
                pointer_mode: xcb::x::GrabMode::Async,
                keyboard_mode: xcb::x::GrabMode::Async,
            };
        }
        // xcb docs: https://www.mankier.com/3/xcb_change_window_attributes
        // xcb::change_window_attributes(&self.conn, root, EVENT_MASK);
        let change_window_attributes = xcb::x::ChangeWindowAttributes{
            window : root,
            // value_list : &'a [Cw],
            value_list :&[
            x::Cw::BackPixel(screen.white_pixel()),
            x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS)],
        };
        self.conn.send_request(&change_window_attributes);
        self.conn.flush().unwrap();
    }

    fn update_screen_dimensions(&mut self) {
        let screen = match self.conn.get_setup().roots().nth(0) {
            None => die!("unable to get handle for screen"),
            Some(s) => s,
        };

        let win_id = self.conn.generate_id();
        let root = screen.root();

        // xcb docs: https://www.mankier.com/3/xcb_create_window
        xcb::x::CreateWindow{
            depth: 0,
            wid: win_id,
            parent: root,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: x::WindowClass::CopyFromParent,
            visual: 0,
            value_list: &[],
        };

    }
    // xcb docs: https://www.mankier.com/3/xcb_input_device_key_press_event_t
    fn key_press(&mut self, event: &xcb::x::KeyPressEvent, bindings: &KeyBindings) {
        log!("handling keypress: {:?} {:?}", event.state(), event.detail());

        if let Some(action) = bindings.get(&KeyCode::from_key_press(event)) {
            log!("running action");
            action(self);
            log!("action run");
        }
    }

    pub fn run(&mut self) {
        let bindings = key_bindings();
        self.grab_keys(&bindings);
        loop {
            match self.conn.wait_for_event().unwrap() {
                xcb::Event::X(x::Event::KeyPress(ev)) => {
                    log!("got event");
                    self.key_press(&ev, &bindings);
                }
                _ => (),

            }

            self.conn.flush().unwrap();
        }
    }

    pub fn kill(&mut self) {
        self.conn.flush().unwrap();
        process::exit(0);
    }
}
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
                mask: mask,
                code: *code,
            })
        }
        None => None,
    }
}


// pub type LayoutFunc = Box<dyn Fn(usize, &Region, usize, f32) -> Vec<Region>>;
pub type FireAndForget = Box<dyn Fn(&mut WindowManager) -> ()>;
pub type KeyBindings = HashMap<KeyCode, FireAndForget>;
pub type CodeMap = HashMap<String, u8>;


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
pub fn key_bindings() -> KeyBindings {
    gen_keybindings! {
        "M-semicolon" => run_external!("rofi-apps"),
        "M-Return" => run_external!("alacritty"),
        "M-S-Escape" => run_internal!(kill);
    }
}
