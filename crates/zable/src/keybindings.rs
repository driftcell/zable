use gpui::App;
use gpui::KeyBinding;

use crate::actions::Quit;

pub fn init(cx: &mut App) {
    // MacOS
    #[cfg(target_os = "macos")]
    {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    }

    // Windows
    #[cfg(target_os = "windows")]
    {
        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
    }
}
