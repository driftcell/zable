use gpui::SharedString;
use gpui_component::IconNamed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZableIcon {
    Plug,
    Plus,
    CircleCheck,
    CircleAlert,
    Check,
    Loading,
    Database,
    Sun,
    Moon,
}

impl IconNamed for ZableIcon {
    fn path(self) -> SharedString {
        match self {
            ZableIcon::Plug => "icons/plug.svg",
            ZableIcon::Plus => "icons/plus.svg",
            ZableIcon::CircleCheck => "icons/circle-check.svg",
            ZableIcon::CircleAlert => "icons/circle-alert.svg",
            ZableIcon::Check => "icons/check.svg",
            ZableIcon::Loading => "icons/loading.svg",
            ZableIcon::Database => "icons/database.svg",
            ZableIcon::Sun => "icons/sun.svg",
            ZableIcon::Moon => "icons/moon.svg",
        }
        .into()
    }
}
