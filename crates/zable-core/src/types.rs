use std::collections::HashMap;

use serde::Serialize;

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

    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        match self {
            DatabaseType::Postgres => "PostgreSQL",
            DatabaseType::MySql => "MySQL",
            DatabaseType::Other(s) => s.as_str(),
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionConfig {
    pub database_type: DatabaseType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub database: Option<String>,
    pub query_params: HashMap<String, String>,
}

impl ConnectionConfig {
    pub fn parse(url: &str) -> Result<Self, url::ParseError> {
        let parsed = url::Url::parse(url)?;

        Ok(Self {
            database_type: DatabaseType::from_schema(parsed.scheme()),
            host: parsed.host_str().map(String::from),
            port: parsed.port().map(|p| p.to_string()),
            username: {
                let u = parsed.username();
                if u.is_empty() {
                    None
                } else {
                    Some(String::from(u))
                }
            },
            password: parsed.password().map(String::from),
            database: {
                let db = parsed.path().trim_start_matches('/');
                if db.is_empty() {
                    None
                } else {
                    Some(String::from(db))
                }
            },
            query_params: parsed
                .query_pairs()
                .map(|(k, v)| (String::from(k.as_ref()), String::from(v.as_ref())))
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
    pub fn has_info(&self) -> bool {
        self.host.is_some()
            || self.port.is_some()
            || self.username.is_some()
            || self.database.is_some()
            || !self.query_params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_info() {
        let config =
            ConnectionConfig::parse("postgres://user:pass@localhost:5432/db?param=value").unwrap();
        assert!(config.has_info());
    }

    #[test]
    fn test_has_no_info() {
        let config = ConnectionConfig::parse("postgres://").unwrap();
        assert!(!config.has_info());
    }
}
