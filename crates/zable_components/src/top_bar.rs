use gpui::{IntoElement, ParentElement, RenderOnce, Styled, px};
use gpui_component::{ActiveTheme, Icon, Theme, ThemeMode, button::Button, h_flex};

use crate::icons::ZableIcon;

const TRAFFIC_LIGHT_WIDTH: f32 = 80.;
const TRAFFIC_LIGHT_HEIGHT: f32 = 4.5;

#[derive(IntoElement)]
pub struct TopBar;

impl RenderOnce for TopBar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme().colors;
        h_flex()
            .pl(px(TRAFFIC_LIGHT_WIDTH))
            .pt(px(TRAFFIC_LIGHT_HEIGHT))
            .justify_between()
            .child(
                h_flex()
                    .gap_2()
                    .child(Icon::new(ZableIcon::Database))
                    .child("Zable")
                    .child(
                        h_flex()
                            .text_color(theme.muted_foreground)
                            .gap_2()
                            .child("/")
                            .child("Connection"),
                    ),
            )
            .child(
                Button::new("toggle_dark")
                    .icon(if cx.theme().is_dark() {
                        Icon::new(ZableIcon::Sun)
                    } else {
                        Icon::new(ZableIcon::Moon)
                    })
                    .on_click(|_, window, cx| {
                        if cx.theme().is_dark() {
                            Theme::change(ThemeMode::Light, Some(window), cx);
                        } else {
                            Theme::change(ThemeMode::Dark, Some(window), cx);
                        }
                    }),
            )
    }
}
