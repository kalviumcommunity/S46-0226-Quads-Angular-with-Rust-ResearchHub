use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEnvironment {
    Development,
    Production,
}

#[derive(Clone)]
pub struct AppConfig {
    pub environment: AppEnvironment,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_acquire_timeout_secs: u64,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let environment = match env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" => AppEnvironment::Production,
            _ => AppEnvironment::Development,
        };

        let host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("APP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            if environment == AppEnvironment::Production {
                panic!("DATABASE_URL is required in production")
            } else {
                "postgres://postgres:postgres@localhost:5432/researchhub".to_string()
            }
        });
        let database_max_connections = env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(if environment == AppEnvironment::Production {
                20
            } else {
                10
            });
        let database_acquire_timeout_secs = env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(if environment == AppEnvironment::Production {
                10
            } else {
                5
            });
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-secret".to_string());

        Self {
            environment,
            host,
            port,
            database_url,
            database_max_connections,
            database_acquire_timeout_secs,
            jwt_secret,
        }
    }
}
