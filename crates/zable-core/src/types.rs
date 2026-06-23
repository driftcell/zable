use std::collections::HashMap;

use anyhow::Result;
use percent_encoding::percent_decode_str;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ConnUrlError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),

    #[error("Missing host")]
    MissingHost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum DatabaseType {
    #[default]
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

    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        match self {
            DatabaseType::Postgres => "PostgreSQL",
            DatabaseType::MySql => "MySQL",
            DatabaseType::Other(s) => s.as_str(),
        }
    }
}

#[derive(Serialize, Default, Clone, Debug)]
pub struct ConnectionConfig {
    pub database_type: DatabaseType,
    pub username: String,
    pub password: Option<String>,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub query_params: HashMap<String, String>,
}

impl ConnectionConfig {
    pub fn parse(url: &str) -> Result<Self> {
        let parsed = url::Url::parse(url)?;

        let database_type = DatabaseType::from_schema(parsed.scheme());
        let host = parsed
            .host_str()
            .ok_or(ConnUrlError::MissingHost)?
            .to_string();
        let port = parsed.port().unwrap_or(5432);
        let username = parsed.username().to_string();
        let password = parsed
            .password()
            .map(|p| percent_decode_str(p).decode_utf8().unwrap().to_string());
        let database = {
            let db = parsed.path().trim_start_matches('/');
            match db.is_empty() {
                true => "postgres".into(),
                false => String::from(db),
            }
        };
        let query_params = parsed
            .query_pairs()
            .map(|(k, v)| (String::from(k.as_ref()), String::from(v.as_ref())))
            .collect();

        Ok(Self {
            database_type,
            host,
            port,
            username,
            password,
            database,
            query_params,
        })
    }
}
