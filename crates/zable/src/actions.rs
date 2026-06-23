use gpui::{App, actions};

actions!(zable, [Quit]);

pub fn init(cx: &mut App) {
    cx.on_action(|_quit: &Quit, cx| cx.quit());
}
