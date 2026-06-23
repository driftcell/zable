use gpui::{App, Menu, MenuItem};

use crate::actions::Quit;

pub fn init(cx: &mut App) {
    cx.set_menus(vec![
        Menu::new("Zable").items(vec![MenuItem::action("Quit Zable", Quit)]),
    ]);
}
