use gpui::*;

use anyhow::anyhow;
use gpui_component::{IconName, Root, WindowExt, button::Button};
use rust_embed::RustEmbed;
use std::borrow::Cow;
use zable_connection_ui::ConnectionView;

actions!(zable, [Quit]);

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(
                Button::new("open_connection_config")
                    .label("New Connection")
                    .icon(IconName::Plus)
                    .on_click(cx.listener(|_this, _, window, cx| {
                        let view = cx.new(|cx| ConnectionView::new(window, cx));
                        window.open_dialog(cx, move |dialog, _window, _cx| {
                            dialog
                                .title("New Connection")
                                .width(px(640.))
                                .child(view.clone())
                        });
                    })),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);

            cx.on_action(|_quit: &Quit, cx| cx.quit());
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus(vec![
                Menu::new("Zable").items(vec![MenuItem::action("Quit Zable", Quit)]),
            ]);

            cx.spawn(async move |cx| {
                cx.open_window(
                    WindowOptions {
                        titlebar: Some(TitlebarOptions {
                            appears_transparent: true,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|_cx| RootView);
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("Failed to open window");
            })
            .detach();
        });
}
