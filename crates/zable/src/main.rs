use gpui::*;

mod views;

use anyhow::anyhow;
use gpui_component::{IconName, Root, button::Button};
use rust_embed::RustEmbed;
use std::borrow::Cow;

use crate::views::ConnectionView;

/// An asset source that loads assets from the `./assets` folder.
#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

struct RootView;

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            Button::new("open_connection_config")
                .label("New Connection")
                .icon(IconName::Plus)
                .on_click(|_, _, cx| {
                    let bounds = WindowBounds::centered(Size::new(px(640.), px(400.)), cx);
                    let titlebar_option = TitlebarOptions {
                        title: Some("New Connection".into()),
                        ..Default::default()
                    };
                    cx.open_window(
                        WindowOptions {
                            window_bounds: Some(bounds),
                            titlebar: Some(titlebar_option),
                            ..Default::default()
                        },
                        |window, cx| {
                            let view = cx.new(|cx| ConnectionView::new(window, cx));
                            cx.new(|cx| Root::new(view, window, cx))
                        },
                    )
                    .expect("Failed to open window");
                }),
        )
    }
}
fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            cx.spawn(async move |cx| {
                cx.open_window(WindowOptions::default(), |window, cx| {
                    let view = cx.new(|_cx| RootView);
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("Failed to open window");
            })
            .detach();
        });
}
