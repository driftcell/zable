use zable_components::{field::Field, icons::ZableIcon};
use zable_core::{
    Tokio,
    postgres::{PgServerInfo, check_pg_connection},
};

use gpui::{
    AppContext, Context, Entity, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    separator::Separator,
    v_flex,
};
use zable_core::ConnectionConfig;

pub struct ConnectionView {
    name_input: Entity<InputState>,
    url_input: Entity<InputState>,
    label_input: Entity<InputState>,
    config: ConnectionConfig,
    parse_error: Option<SharedString>,
    is_testing: bool,
    test_result: Option<PgServerInfo>,
}

impl ConnectionView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("My Database"));
        let url_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("postgresql://user:pass@localhost:5432/mydb?sslmode=require")
        });

        let label_input = cx.new(|cx| InputState::new(window, cx).placeholder("Production"));

        cx.subscribe_in(&url_input, window, |this, input, event, _window, cx| {
            if let InputEvent::Change = event {
                let raw = input.read(cx).value();

                if raw.is_empty() {
                    this.parse_error = None;
                    cx.notify();
                    return;
                }

                match ConnectionConfig::parse(&raw) {
                    Ok(cfg) => {
                        this.config = cfg;
                        this.parse_error = None;
                    }
                    Err(e) => {
                        this.parse_error = Some(SharedString::from(e.to_string()));
                    }
                }
                cx.notify();
            }
        })
        .detach();

        Self {
            name_input,
            url_input,
            label_input,
            config: ConnectionConfig::default(),
            parse_error: None,
            is_testing: false,
            test_result: None,
        }
    }

    fn handle_test_connection(&mut self, cx: &mut Context<Self>) {
        let config = self.config.clone();

        let task = Tokio::spawn(cx, async move { check_pg_connection(&config).await });

        self.is_testing = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = task.await;

            this.update(cx, |this, cx| {
                this.is_testing = false;
                this.test_result = {
                    match outcome {
                        Ok(Ok(info)) => Some(info),
                        Ok(Err(_)) => None,
                        Err(_) => None,
                    }
                };
                cx.notify();
            })
        })
        .detach();
    }
}

impl Render for ConnectionView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // Copy the colors out so we don't hold an immutable borrow of `cx`
        // across calls that need `&mut cx`.
        let theme = cx.theme().colors;
        let has_error = self.parse_error.is_some();

        // Header
        let header = v_flex()
            .items_start()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .size_8()
                    .flex_shrink_0()
                    .rounded_md()
                    .child(Icon::new(ZableIcon::Plug).size_4())
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("New Connection"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Configure a database connection"),
            );

        // Name + Label row
        let id_row = h_flex()
            .gap_4()
            .child(v_flex().flex_1().child(Field::new(
                "Name",
                "Friendly Name",
                Input::new(&self.name_input),
                theme,
            )))
            .child(v_flex().flex_1().child(Field::new(
                "Label",
                "Group · Optional",
                Input::new(&self.label_input),
                theme,
            )));

        // URL row with a leading icon
        let url_input = Input::new(&self.url_input);

        let url_field = v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        Label::new("Connection URL")
                            .text_sm()
                            .font_medium()
                            .text_color(theme.foreground),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Connection String"),
                    ),
            )
            .child(url_input);

        // Footer actions
        let mut footer_left = h_flex().items_center().gap_1p5();
        if has_error {
            footer_left = footer_left
                .child(
                    Icon::new(ZableIcon::CircleAlert)
                        .size_3p5()
                        .text_color(theme.danger),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.danger)
                        .child("Please fix the error before saving."),
                );
        } else {
            footer_left = footer_left
                .child(
                    Icon::new(ZableIcon::CircleCheck)
                        .size_3p5()
                        .text_color(theme.success),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("URL is valid."),
                )
                .when_some(self.test_result.as_ref(), |this, test_result| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("{}ms", test_result.elapsed.as_millis())),
                    )
                });
        }

        let footer_actions = h_flex().items_center().gap_2().child(
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("cancel")
                        .label("Cancel")
                        .on_click(|_, window, cx| {
                            window.close_dialog(cx);
                        }),
                )
                .child(
                    Button::new("test")
                        .label("Test")
                        .outline()
                        .icon(ZableIcon::Plug)
                        .secondary()
                        .disabled(self.is_testing)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.handle_test_connection(cx);
                        })),
                )
                .child(
                    Button::new("save")
                        .label("Save")
                        .primary()
                        .on_click(|_, _, _| {}),
                ),
        );

        let footer = h_flex()
            .items_center()
            .justify_between()
            .child(footer_left)
            .child(footer_actions);

        // Assemble the card
        v_flex()
            .min_w(px(560.))
            .max_w(px(640.))
            .p_6()
            .gap_5()
            .bg(theme.background)
            .child(header)
            .child(Separator::horizontal())
            .child(id_row)
            .child(url_field)
            .child(Separator::horizontal())
            .child(footer)
    }
}
