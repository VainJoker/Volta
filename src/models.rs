use xcb::randr::NotifyMask;
use std::collections::HashMap;
use xcb::{self, x::{self,KeyButMask}};
use crate::config;
use std::process;

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
        let bindings = config::key_bindings();
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
