use std::collections::HashMap;

use gpui::{AppContext, Context, Entity, ParentElement, Render, SharedString, Styled, Window, div};
use gpui_component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    v_flex,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DatabaseType {
    Postgres,
    MySql,
    Redis,
    Mongodb,
    Other(String),
}

impl DatabaseType {
    fn from_schema(schema: &str) -> Self {
        match schema {
            "postgres" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySql,
            "redis" => DatabaseType::Redis,
            "mongodb" => DatabaseType::Mongodb,
            _ => DatabaseType::Other(schema.to_string()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            DatabaseType::Postgres => "postgres",
            DatabaseType::MySql => "mysql",
            DatabaseType::Redis => "redis",
            DatabaseType::Mongodb => "mongodb",
            DatabaseType::Other(s) => s.as_str(),
        }
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
}

pub struct ConnectionView {
    url_input: Entity<InputState>,
    config: ConnectionConfig,
    parse_error: Option<SharedString>,
}

impl ConnectionView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("postgresql://user:pass@localhost:5432/mydb?sslmode=require")
        });

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
            url_input,
            config: ConnectionConfig::empty(),
            parse_error: None,
        }
    }
}

/// A label-value row for displaying a single config field.
fn field_row(
    label: impl Into<SharedString>,
    value: Option<impl Into<SharedString>>,
    masked: bool,
) -> impl gpui::IntoElement {
    let value = value
        .map(|v| v.into())
        .unwrap_or_else(|| SharedString::from("—"));
    h_flex()
        .gap_3()
        .child(div().w_32().child(Label::new(label)))
        .child(Label::new(value).masked(masked))
}

impl Render for ConnectionView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let config = &self.config;
        let type_str = config.database_type.as_str();

        v_flex()
            .gap_4()
            .size_full()
            .p_6()
            .child(
                v_flex()
                    .gap_2()
                    .child(Label::new("Connection URL"))
                    .child(Input::new(&self.url_input).cleanable(true)),
            )
            .children(self.parse_error.as_ref().map(|e| Label::new(e.clone())))
            .child(
                v_flex()
                    .gap_2()
                    .child(Label::new("Parsed Configuration"))
                    .child(field_row(
                        "Type",
                        (!type_str.is_empty()).then(|| type_str.to_string()),
                        false,
                    ))
                    .child(field_row("Host", config.host.clone(), false))
                    .child(field_row("Port", config.port.clone(), false))
                    .child(field_row("Username", config.username.clone(), false))
                    .child(field_row("Password", config.password.clone(), true))
                    .child(field_row("Database", config.database.clone(), false))
                    .children(
                        config
                            .query_params
                            .iter()
                            .map(|(k, v)| field_row(k.clone(), Some(v.clone()), false)),
                    ),
            )
    }
}
