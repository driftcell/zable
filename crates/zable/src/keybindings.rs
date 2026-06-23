use gpui::App;
use gpui::KeyBinding;

use crate::actions::Quit;

pub fn init(cx: &mut App) {
    // MacOS
    #[cfg(target_os = "macos")]
    {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    }

    // Windows & Linux
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
    }
}
