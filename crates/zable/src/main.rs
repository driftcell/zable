mod actions;
mod assets;
mod keybindings;
mod menus;

use gpui::*;

use gpui_component::{Root, WindowExt, button::Button};
use zable_components::{icons::ZableIcon, top_bar::TopBar};
use zable_connection_ui::ConnectionView;

use crate::assets::Assets;

struct RootView;

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(TopBar)
            .child(
                Button::new("open_connection_config")
                    .label("New Connection")
                    .icon(ZableIcon::Plus)
                    .on_click(cx.listener(|_this, _, window, cx| {
                        let view = cx.new(|cx| ConnectionView::new(window, cx));
                        window.open_dialog(cx, move |dialog, _window, _cx| {
                            dialog.width(px(640.)).child(view.clone())
                        });
                    })),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);

            zable_core::init(cx);
            actions::init(cx);
            keybindings::init(cx);
            menus::init(cx);

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
