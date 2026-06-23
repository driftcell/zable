use gpui::SharedString;
use gpui_component::IconNamed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZableIcon {
    Plug,
    Plus,
    CircleCheck,
    CircleAlert,
}

impl IconNamed for ZableIcon {
    fn path(self) -> SharedString {
        match self {
            ZableIcon::Plug => "icons/plug.svg",
            ZableIcon::Plus => "icons/plus.svg",
            ZableIcon::CircleCheck => "icons/circle-check.svg",
            ZableIcon::CircleAlert => "icons/circle-alert.svg",
        }
        .into()
    }
}
