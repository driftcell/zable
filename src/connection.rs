use std::collections::HashMap;

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    div, px,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, IconNamed, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    separator::Separator,
    v_flex,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionIcon {
    Plug,
}

impl IconNamed for ConnectionIcon {
    fn path(self) -> SharedString {
        match self {
            ConnectionIcon::Plug => "icons/plug.svg",
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Other(String),
}

impl DatabaseType {
    fn from_schema(schema: &str) -> Self {
        match schema {
            "postgres" | "postgresql" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySql,
            _ => DatabaseType::Other(schema.to_string()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            DatabaseType::Postgres => "PostgreSQL",
            DatabaseType::MySql => "MySQL",
            DatabaseType::Other(s) => s.as_str(),
        }
    }

    /// A representative icon for the database type.
    fn icon(&self) -> IconName {
        IconName::HardDrive
    }
}

#[derive(Serialize)]
pub struct ConnectionConfig {
    pub database_type: DatabaseType,
    pub username: Option<SharedString>,
    pub password: Option<SharedString>,
    pub host: Option<SharedString>,
    pub port: Option<SharedString>,
    pub database: Option<SharedString>,
    pub query_params: HashMap<SharedString, SharedString>,
}

impl ConnectionConfig {
    pub fn parse(url: &str) -> Result<Self, url::ParseError> {
        let parsed = url::Url::parse(url)?;

        Ok(Self {
            database_type: DatabaseType::from_schema(parsed.scheme()),
            host: parsed.host_str().map(SharedString::from),
            port: parsed.port().map(|p| p.to_string()).map(SharedString::from),
            username: {
                let u = parsed.username();
                if u.is_empty() {
                    None
                } else {
                    Some(SharedString::from(u))
                }
            },
            password: parsed.password().map(SharedString::from),
            database: {
                let db = parsed.path().trim_start_matches('/');
                if db.is_empty() {
                    None
                } else {
                    Some(SharedString::from(db))
                }
            },
            query_params: parsed
                .query_pairs()
                .map(|(k, v)| {
                    (
                        SharedString::from(k.as_ref()),
                        SharedString::from(v.as_ref()),
                    )
                })
                .collect(),
        })
    }

    /// Empty placeholder for when no URL has been entered yet.
    pub fn empty() -> Self {
        Self {
            database_type: DatabaseType::Other(String::new()),
            username: None,
            password: None,
            host: None,
            port: None,
            database: None,
            query_params: HashMap::new(),
        }
    }

    /// Whether any meaningful field has been parsed from the URL.
    fn has_info(&self) -> bool {
        self.host.is_some()
            || self.port.is_some()
            || self.username.is_some()
            || self.database.is_some()
            || !self.query_params.is_empty()
    }
}

pub struct ConnectionView {
    name_input: Entity<InputState>,
    url_input: Entity<InputState>,
    label_input: Entity<InputState>,
    config: ConnectionConfig,
    parse_error: Option<SharedString>,
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
            config: ConnectionConfig::empty(),
            parse_error: None,
        }
    }

    /// A single labeled input row, using the library's `Label` for the
    /// caption and a muted hint underneath.
    fn field(
        label: &'static str,
        hint: &'static str,
        input: impl IntoElement,
        theme: gpui_component::ThemeColor,
    ) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .justify_between()
                    .child(
                        Label::new(label)
                            .text_sm()
                            .font_medium()
                            .text_color(theme.foreground),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(hint),
                    ),
            )
            .child(input)
    }

    /// A small pill showing a key/value pair parsed from the connection URL.
    fn info_pill(
        key: &'static str,
        value: SharedString,
        theme: gpui_component::ThemeColor,
    ) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_md()
            .bg(theme.secondary)
            .text_xs()
            .child(div().text_color(theme.muted_foreground).child(key))
            .child(
                div()
                    .text_color(theme.foreground)
                    .font_medium()
                    .child(value),
            )
    }
}

impl Render for ConnectionView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        // Copy the colors out so we don't hold an immutable borrow of `cx`
        // across calls that need `&mut cx`.
        let theme = cx.theme().colors;
        let config = &self.config;
        let has_error = self.parse_error.is_some();
        let has_info = config.has_info() && !has_error;

        // Header
        let header = h_flex()
            .items_center()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .justify_center()
                    .size_8()
                    .flex_shrink_0()
                    .rounded_md()
                    .bg(theme.primary.opacity(0.12))
                    .text_color(theme.primary)
                    .child(Icon::new(ConnectionIcon::Plug).size_4()),
            )
            .child(
                v_flex()
                    .gap_0p5()
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("New Connection"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Configure a database connection"),
                    ),
            );

        // Name + Label row
        let id_row = h_flex()
            .gap_4()
            .child(v_flex().flex_1().child(Self::field(
                "Name",
                "Friendly Name",
                Input::new(&self.name_input),
                theme,
            )))
            .child(v_flex().flex_1().child(Self::field(
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

        // Live preview / status area
        let preview = if has_error {
            let err = self.parse_error.clone().unwrap_or_default();
            h_flex()
                .items_start()
                .gap_2()
                .p_3()
                .rounded_md()
                .bg(theme.danger.opacity(0.1))
                .border_1()
                .border_color(theme.danger.opacity(0.3))
                .child(
                    Icon::new(IconName::TriangleAlert)
                        .size_4()
                        .flex_shrink_0()
                        .text_color(theme.danger),
                )
                .child(
                    v_flex()
                        .gap_0p5()
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(theme.danger)
                                .child("Invalid URL"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(err),
                        ),
                )
        } else if has_info {
            let db_type = SharedString::from(config.database_type.as_str());
            let db_icon = config.database_type.icon();
            let mut body = h_flex().items_start().gap_3().flex_wrap();

            // Database type chip
            body = body.child(
                h_flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(theme.primary.opacity(0.12))
                    .child(Icon::new(db_icon).size_4().text_color(theme.primary))
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(theme.primary)
                            .child(db_type),
                    ),
            );

            // Detail pills
            let mut details = v_flex().gap_1();

            let mut first_row = h_flex().gap_2().flex_wrap();

            if let Some(host) = config.host.clone() {
                first_row = first_row.child(Self::info_pill("host", host, theme));
            }
            if let Some(port) = config.port.clone() {
                first_row = first_row.child(Self::info_pill("port", port, theme));
            }
            if let Some(db) = config.database.clone() {
                first_row = first_row.child(Self::info_pill("db", db, theme));
            }
            details = details.child(first_row);

            let mut second_row = h_flex().gap_2().flex_wrap();
            let mut has_second = false;
            if let Some(user) = config.username.clone() {
                second_row = second_row.child(Self::info_pill("user", user, theme));
                has_second = true;
            }
            if !config.query_params.is_empty() {
                second_row = second_row.child(Self::info_pill(
                    "params",
                    SharedString::from(format!("{} item(s)", config.query_params.len())),
                    theme,
                ));
                has_second = true;
            }
            if has_second {
                details = details.child(second_row);
            }

            body = body.child(
                details.child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(Icon::new(IconName::CircleCheck).size_3())
                        .child("Parsed from URL"),
                ),
            );

            body.p_3()
                .rounded_md()
                .bg(theme.secondary.opacity(0.5))
                .border_1()
                .border_color(theme.border)
        } else {
            h_flex()
                .items_center()
                .gap_2()
                .p_3()
                .rounded_md()
                .bg(theme.muted.opacity(0.4))
                .child(
                    Icon::new(IconName::Info)
                        .size_4()
                        .flex_shrink_0()
                        .text_color(theme.muted_foreground),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("Enter a valid connection URL to see a live preview."),
                )
        };

        // Footer actions
        let mut footer_left = h_flex().items_center().gap_1p5();
        if has_error {
            footer_left = footer_left
                .child(
                    Icon::new(IconName::TriangleAlert)
                        .size_3p5()
                        .text_color(theme.danger),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.danger)
                        .child("Please fix the error before saving."),
                );
        } else if has_info {
            footer_left = footer_left
                .child(
                    Icon::new(IconName::CircleCheck)
                        .size_3p5()
                        .text_color(theme.success),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("URL is valid."),
                );
        }

        let footer_actions = h_flex().items_center().gap_2().child(
            h_flex()
                .items_center()
                .gap_2()
                .child(Button::new("cancel").label("Cancel").ghost())
                .child(
                    Button::new("test")
                        .label("Test")
                        .outline()
                        .icon(ConnectionIcon::Plug),
                )
                .child(Button::new("save").label("Save").primary()),
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
            .child(preview)
            .child(Separator::horizontal())
            .child(footer)
    }
}
