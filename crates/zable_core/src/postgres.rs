use anyhow::Result;
use std::time::{Duration, Instant};
use tokio_postgres::error::SqlState;
use tokio_postgres::{Config, NoTls};

use crate::ConnectionConfig;

#[derive(Debug, Clone)]
pub struct PgServerInfo {
    pub version: String,
    pub elapsed: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum TestConnError {
    #[error("Connection timeout (exceeded {0:?})")]
    Timeout(Duration),

    #[error("Authentication failed: username or password is incorrect")]
    Auth,

    #[error("Database \"{0}\" does not exist")]
    DatabaseNotFound(String),

    #[error(transparent)]
    Other(tokio_postgres::Error),
}

fn classify(err: tokio_postgres::Error, cfg: &ConnectionConfig) -> TestConnError {
    let Some(code) = err.code() else {
        return TestConnError::Other(err);
    };

    if *code == SqlState::INVALID_PASSWORD || *code == SqlState::INVALID_AUTHORIZATION_SPECIFICATION
    {
        TestConnError::Auth
    } else if *code == SqlState::INVALID_CATALOG_NAME {
        TestConnError::DatabaseNotFound(cfg.database.clone())
    } else {
        TestConnError::Other(err)
    }
}

pub async fn check_pg_connection(cfg: &ConnectionConfig) -> Result<PgServerInfo> {
    let mut config = Config::new();
    config
        .host(&cfg.host)
        .port(cfg.port)
        .user(&cfg.username)
        .dbname(&cfg.database)
        .connect_timeout(Duration::from_secs(5))
        .application_name("zable");

    if let Some(password) = &cfg.password {
        config.password(password);
    }

    let started = Instant::now();

    let connect_fut = config.connect(NoTls);
    let (client, connection) = match tokio::time::timeout(Duration::from_secs(5), connect_fut).await
    {
        Err(_elapsed) => return Err(TestConnError::Timeout(Duration::from_secs(5)).into()),
        Ok(Err(err)) => return Err(classify(err, cfg).into()),
        Ok(Ok(pair)) => pair,
    };

    let conn_task = tokio::spawn(async move {
        let _ = connection.await;
    });

    let version: String = match client
        .query_one("SELECT current_setting('server_version')", &[])
        .await
    {
        Ok(row) => row.get(0),
        Err(err) => {
            conn_task.abort();
            return Err(classify(err, cfg).into());
        }
    };

    // 收尾:drop(client) 会让 connection future 自然结束,conn_task 随之完成
    drop(client);
    let _ = conn_task.await;

    Ok(PgServerInfo {
        version,
        elapsed: started.elapsed(),
    })
}
