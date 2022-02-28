use crate::models::KeyBindings;
pub fn key_bindings() -> KeyBindings {
    gen_keybindings! {
        "M-semicolon" => run_external!("firefox"),
        "M-Return" => run_external!("alacritty"),
        "M-S-Escape" => run_internal!(kill);
    }
}
