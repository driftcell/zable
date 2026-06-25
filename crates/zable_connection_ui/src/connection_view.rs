use gpui::{
    AppContext, Context, Entity, Hsla, IntoElement, ParentElement, Render, SharedString,
    Styled as _, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, StyledExt as _, ThemeColor, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    separator::Separator,
    v_flex,
};
use zable_components::{field::Field, icons::ZableIcon};
use zable_core::{
    ConnectionConfig, Tokio,
    config::{AppConfig, ConnectionEntry},
    postgres::{PgServerInfo, check_pg_connection},
};

#[derive(Debug)]
enum UrlStatus {
    Empty,
    Invalid,
    Valid(ValidConnection),
}

#[derive(Debug)]
struct ValidConnection {
    connection_config: ConnectionConfig,
    test_status: TestStatus,
}

#[derive(Debug)]
enum TestStatus {
    Idle,
    Testing,
    Tested(PgServerInfo),
    Failed(SharedString),
}

#[derive(Debug)]
struct StatusHint {
    icon: ZableIcon,
    color: Hsla,
    text: SharedString,
}

pub struct ConnectionView {
    url_input: Entity<InputState>,
    name_input: Entity<InputState>,
    label_input: Entity<InputState>,
    url_status: UrlStatus,
}

impl ConnectionView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("postgres://postgres:password@localhost:5432/postgres")
        });
        let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("Postgres"));
        let label_input = cx.new(|cx| InputState::new(window, cx).placeholder("dev"));

        cx.subscribe_in(&url_input, window, |this, input, event, _window, cx| {
            if let InputEvent::Change = event {
                let raw = input.read(cx).value();

                match ConnectionConfig::parse(&raw) {
                    Ok(config) => {
                        this.url_status = UrlStatus::Valid(ValidConnection {
                            connection_config: config,
                            test_status: TestStatus::Idle,
                        })
                    }
                    Err(_) => this.url_status = UrlStatus::Invalid,
                }

                cx.notify();
            }
        })
        .detach();

        Self {
            url_input,
            name_input,
            label_input,
            url_status: UrlStatus::Empty,
        }
    }
}

impl ConnectionView {
    fn render_header(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().colors;

        v_flex()
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
            )
    }

    fn render_body(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().colors;
        v_flex()
            .gap_4()
            .child(
                h_flex()
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
                    ))),
            )
            .child(
                v_flex()
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
                    .child(Input::new(&self.url_input)),
            )
    }

    fn render_footer_left(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme().colors;
        h_flex()
            .items_center()
            .gap_1p5()
            .when_some(self.status_hint(&theme), |this, hint| {
                this.child(Icon::new(hint.icon).size_2p5().text_color(hint.color))
                    .child(div().text_xs().text_color(hint.color).child(hint.text))
            })
    }

    fn render_footer_actions(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("cancel")
                    .label("Cancel")
                    .on_click(|_, window, cx| window.close_dialog(cx)),
            )
            .child(
                Button::new("test")
                    .outline()
                    .secondary()
                    .label("Test")
                    .map(|this| match &self.url_status {
                        UrlStatus::Empty => this.icon(ZableIcon::Plug).disabled(true),
                        UrlStatus::Valid(conn) => match &conn.test_status {
                            TestStatus::Idle | TestStatus::Failed(_) => {
                                this.label("Test").icon(ZableIcon::Plug)
                            }
                            TestStatus::Testing => this
                                .label("Testing")
                                .disabled(true)
                                .loading(true)
                                .loading_icon(ZableIcon::Loading),
                            TestStatus::Tested(_) => this
                                .label("Tested")
                                .icon(ZableIcon::CircleCheck)
                                .disabled(true),
                        },
                        _ => this.icon(ZableIcon::Plug),
                    })
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.handle_test_connection(window, cx);
                    })),
            )
            .child(
                Button::new("save")
                    .label("Save")
                    .primary()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.handle_save_config(window, cx);
                    })),
            )
    }

    fn render_footer(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_between()
            .child(self.render_footer_left(window, cx))
            .child(self.render_footer_actions(window, cx))
    }

    fn handle_test_connection(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let UrlStatus::Valid(config) = &mut self.url_status else {
            return;
        };
        config.test_status = TestStatus::Testing;
        cx.notify();

        let config = config.connection_config.clone();
        let task = Tokio::spawn(cx, async move { check_pg_connection(&config).await });

        cx.spawn(async |this, cx| {
            let result = task.await;

            this.update(cx, |this, cx| {
                let UrlStatus::Valid(valid) = &mut this.url_status else {
                    return;
                };

                valid.test_status = match result {
                    Ok(Ok(info)) => TestStatus::Tested(info),
                    Ok(Err(e)) => TestStatus::Failed(e.to_string().into()),
                    Err(e) => TestStatus::Failed(e.to_string().into()),
                };

                cx.notify();
            })
            .ok()
        })
        .detach();
    }

    fn handle_save_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let UrlStatus::Valid(config) = &self.url_status else {
            return;
        };

        let name = self.name_input.read(cx).value();
        let label = self.label_input.read(cx).value();

        match AppConfig::load() {
            Ok(mut app_config) => {
                app_config.upsert(ConnectionEntry::new(name, label, &config.connection_config));
                if let Err(e) = app_config.save() {
                    eprintln!("Failed to save config: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to save config: {}", e);
            }
        }

        window.close_dialog(cx);
    }
}

impl ConnectionView {
    fn status_hint(&self, theme: &ThemeColor) -> Option<StatusHint> {
        let hint = match &self.url_status {
            UrlStatus::Empty => return None,
            UrlStatus::Invalid => StatusHint {
                icon: ZableIcon::CircleAlert,
                color: theme.danger,
                text: "Please fix the error before saving.".into(),
            },
            UrlStatus::Valid(conn) => match &conn.test_status {
                TestStatus::Idle => StatusHint {
                    icon: ZableIcon::CircleCheck,
                    color: theme.success,
                    text: "URL is valid.".into(),
                },
                TestStatus::Testing => StatusHint {
                    icon: ZableIcon::CircleCheck,
                    color: theme.success,
                    text: "URL is valid.".into(),
                },
                TestStatus::Tested(info) => StatusHint {
                    icon: ZableIcon::Check,
                    color: theme.success,
                    text: format!("{} {}ms", info.version, info.elapsed.as_millis()).into(),
                },
                TestStatus::Failed(e) => StatusHint {
                    icon: ZableIcon::CircleAlert,
                    color: theme.danger,
                    text: e.clone(),
                },
            },
        };
        Some(hint)
    }
}

impl Render for ConnectionView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme().colors;
        v_flex()
            .min_w(px(560.))
            .max_w(px(640.))
            .p_6()
            .gap_5()
            .bg(theme.background)
            .child(self.render_header(window, cx))
            .child(Separator::horizontal())
            .child(self.render_body(window, cx))
            .child(Separator::horizontal())
            .child(self.render_footer(window, cx))
    }
}
