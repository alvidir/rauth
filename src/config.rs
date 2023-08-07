use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use std::env;

const DEFAULT_PORT: &str = "8000";
const DEFAULT_ADDR: &str = "127.0.0.1";
const DEFAULT_TEMPLATES_PATH: &str = "/etc/rauth/smtp/templates/*.html";
const DEFAULT_JWT_HEADER: &str = "authorization";
const DEFAULT_TOTP_HEADER: &str = "x-totp-secret";
const DEFAULT_TOKEN_TIMEOUT: u64 = 7200;
const DEFAULT_TOTP_SECRET_LEN: usize = 32_usize;
#[allow(dead_code)]
const DEFAULT_POOL_SIZE: u32 = 10;
#[allow(dead_code)]
const DEFAULT_CONN_TIMEOUT: u32 = 100; //ms

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_ADDR: &str = "SERVICE_ADDR";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_TOTP_HEADER: &str = "TOTP_HEADER";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";
const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_ISSUER: &str = "SMTP_ISSUER";
const ENV_SMTP_TEMPLATES: &str = "SMTP_TEMPLATES";
const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";
const ENV_PWD_SUFIX: &str = "PWD_SUFIX";
const ENV_TOTP_SECRET_LEN: &str = "TOTP_SECRET_LEN";
const ENV_TOKEN_ISSUER: &str = "TOKEN_ISSUER";

pub static SERVICE_ADDR: Lazy<String> = Lazy::new(|| {
    let netw = env::var(ENV_SERVICE_ADDR).unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let port = env::var(ENV_SERVICE_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
    format!("{}:{}", netw, port)
});

pub static TOKEN_TIMEOUT: Lazy<u64> = Lazy::new(|| {
    env::var(ENV_TOKEN_TIMEOUT)
        .map(|timeout| timeout.parse().unwrap())
        .unwrap_or(DEFAULT_TOKEN_TIMEOUT)
});

pub static JWT_SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
    env::var(ENV_JWT_SECRET)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt secret must be set")
});

pub static JWT_PUBLIC: Lazy<Vec<u8>> = Lazy::new(|| {
    env::var(ENV_JWT_PUBLIC)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt public key must be set")
});

pub static JWT_HEADER: Lazy<String> =
    Lazy::new(|| env::var(ENV_JWT_HEADER).unwrap_or_else(|_| DEFAULT_JWT_HEADER.to_string()));

pub static TOTP_HEADER: Lazy<String> =
    Lazy::new(|| env::var(ENV_TOTP_HEADER).unwrap_or_else(|_| DEFAULT_TOTP_HEADER.to_string()));

pub static SMTP_TRANSPORT: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_TRANSPORT).expect("smtp transport must be set"));

pub static SMTP_USERNAME: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_USERNAME).unwrap_or_default());

pub static SMTP_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_PASSWORD).unwrap_or_default());

pub static SMTP_ORIGIN: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_ORIGIN).expect("smpt origin must be set"));

pub static SMTP_ISSUER: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_ISSUER).expect("smtp issuer must be set"));

pub static SMTP_TEMPLATES: Lazy<String> = Lazy::new(|| {
    env::var(ENV_SMTP_TEMPLATES).unwrap_or_else(|_| DEFAULT_TEMPLATES_PATH.to_string())
});

pub static PWD_SUFIX: Lazy<String> =
    Lazy::new(|| env::var(ENV_PWD_SUFIX).expect("password sufix must be set"));

pub static TOTP_SECRET_LEN: Lazy<usize> = Lazy::new(|| {
    env::var(ENV_TOTP_SECRET_LEN)
        .map(|len| len.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_TOTP_SECRET_LEN)
});

pub static TOKEN_ISSUER: Lazy<String> =
    Lazy::new(|| env::var(ENV_TOKEN_ISSUER).expect("token issuer must be set"));

#[cfg(feature = "postgres")]
pub use postgres::*;

#[cfg(feature = "postgres")]
mod postgres {
    use once_cell::sync::Lazy;
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::{env, time::Duration};

    const ENV_POSTGRES_TIMEOUT: &str = "POSTGRES_TIMEOUT";
    const ENV_POSTGRES_DSN: &str = "POSTGRES_DSN";
    const ENV_POSTGRES_POOL: &str = "POSTGRES_POOL";

    pub static POSTGRES_TIMEOUT: Lazy<Duration> = Lazy::new(|| {
        Duration::from_millis(
            env::var(ENV_POSTGRES_TIMEOUT)
                .unwrap_or(super::DEFAULT_CONN_TIMEOUT.to_string())
                .parse()
                .expect("timeout must be an unsigned number of seconds"),
        )
    });

    pub static POSTGRES_POOL: Lazy<PgPool> = Lazy::new(|| {
        futures::executor::block_on(async {
            let postgres_dsn = env::var(ENV_POSTGRES_DSN).expect("postgres dns must be set");

            let postgres_pool = env::var(ENV_POSTGRES_POOL)
                .map(|pool_size| pool_size.parse().unwrap())
                .unwrap_or(super::DEFAULT_POOL_SIZE);

            PgPoolOptions::new()
                .max_connections(postgres_pool)
                .acquire_timeout(*POSTGRES_TIMEOUT)
                .connect(&postgres_dsn)
                .await
                .unwrap()
        })
    });
}

#[cfg(feature = "redis-cache")]
pub use redis_cache::*;

#[cfg(feature = "redis-cache")]
mod redis_cache {
    use once_cell::sync::Lazy;
    use reool::{config::DefaultCommandTimeout, RedisPool};
    use std::env;
    use tokio::runtime::Handle;

    const ENV_REDIS_TIMEOUT: &str = "REDIS_TIMEOUT";

    const ENV_REDIS_URL: &str = "REDIS_URL";
    const ENV_REDIS_POOL: &str = "REDIS_POOL";

    pub static REDIS_TIMEOUT: Lazy<DefaultCommandTimeout> = Lazy::new(|| {
        env::var(ENV_REDIS_TIMEOUT)
            .unwrap_or(super::DEFAULT_CONN_TIMEOUT.to_string())
            .parse()
            .unwrap()
    });

    pub static REDIS_POOL: Lazy<RedisPool> = Lazy::new(|| {
        let redis_url: String = env::var(ENV_REDIS_URL).expect("redis url must be set");
        let redis_pool: usize = env::var(ENV_REDIS_POOL)
            .map(|pool_size| pool_size.parse().unwrap())
            .unwrap_or_else(|_| super::DEFAULT_POOL_SIZE.try_into().unwrap());

        RedisPool::builder()
            .connect_to_node(redis_url)
            .desired_pool_size(redis_pool)
            .task_executor(Handle::current())
            .default_command_timeout(*REDIS_TIMEOUT)
            .finish_redis_rs()
            .unwrap()
    });
}

#[cfg(feature = "rabbitmq")]
pub use rabbitmq::*;

#[cfg(feature = "rabbitmq")]
mod rabbitmq {
    use deadpool_lapin::{Config, Pool, Runtime, Timeouts};
    use lapin::{options, types::FieldTable, ExchangeKind};
    use once_cell::sync::Lazy;
    use std::env;
    use std::time::Duration;

    const ENV_RABBITMQ_USERS_EXCHANGE: &str = "RABBITMQ_USERS_EXCHANGE";
    const ENV_RABBITMQ_TIMEOUT: &str = "RABBITMQ_TIMEOUT";
    const ENV_RABBITMQ_URL: &str = "RABBITMQ_URL";
    const ENV_RABBITMQ_POOL: &str = "RABBITMQ_POOL";
    const ENV_EVENT_ISSUER: &str = "EVENT_ISSUER";

    pub static RABBITMQ_USERS_EXCHANGE: Lazy<String> = Lazy::new(|| {
        env::var(ENV_RABBITMQ_USERS_EXCHANGE).expect("rabbitmq users bus name must be set")
    });

    pub static RABBITMQ_TIMEOUT: Lazy<Duration> = Lazy::new(|| {
        Duration::from_millis(
            env::var(ENV_RABBITMQ_TIMEOUT)
                .unwrap_or(super::DEFAULT_CONN_TIMEOUT.to_string())
                .parse()
                .expect("timeout must be an unsigned number of seconds"),
        )
    });

    pub static RABBITMQ_POOL: Lazy<Pool> = Lazy::new(|| {
        futures::executor::block_on(async {
            let rabbitmq_url = env::var(ENV_RABBITMQ_URL).expect("rabbitmq url must be set");
            let rabbitmq_pool = env::var(ENV_RABBITMQ_POOL)
                .map(|pool_size| pool_size.parse().unwrap())
                .unwrap_or_else(|_| super::DEFAULT_POOL_SIZE.try_into().unwrap());

            let pool = Config {
                url: Some(rabbitmq_url),
                ..Default::default()
            }
            .builder(Some(Runtime::Tokio1))
            .max_size(rabbitmq_pool)
            .timeouts(Timeouts {
                wait: Some(*RABBITMQ_TIMEOUT),
                create: Some(*RABBITMQ_TIMEOUT),
                recycle: Some(*RABBITMQ_TIMEOUT),
            })
            .build()
            .unwrap();

            let channel = pool.get().await.unwrap().create_channel().await.unwrap();

            let exchange_options = options::ExchangeDeclareOptions {
                durable: true,
                auto_delete: false,
                internal: false,
                nowait: false,
                passive: false,
            };

            channel
                .exchange_declare(
                    &RABBITMQ_USERS_EXCHANGE,
                    ExchangeKind::Fanout,
                    exchange_options,
                    FieldTable::default(),
                )
                .await
                .unwrap();

            pool
        })
    });

    pub static EVENT_ISSUER: Lazy<String> =
        Lazy::new(|| env::var(ENV_EVENT_ISSUER).expect("event issuer must be set"));
}

#[cfg(feature = "trace")]
pub use trace::*;

#[cfg(feature = "trace")]
mod trace {
    use once_cell::sync::Lazy;
    use opentelemetry_api::global;
    use opentelemetry_api::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
    use opentelemetry_semantic_conventions::resource;
    use std::env;
    use tonic::codegen::http::HeaderMap;
    use tonic::metadata::MetadataMap;
    use tracing::metadata::LevelFilter;
    use tracing::subscriber::SetGlobalDefaultError;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Layer;
    use tracing_subscriber::Registry;

    const KEY_VALUE_SEPARATOR: &str = "=";

    const DEFAULT_SERVICE_NAME: &str = "rauth";
    const DEFAULT_HEADERS_SEPARATOR: &str = ";";

    const ENV_SERVICE_NAME: &str = "SERVICE_NAME";
    const ENV_COLLECTOR_URL: &str = "COLLECTOR_URL";
    const ENV_COLLECTOR_HEADERS: &str = "COLLECTOR_HEADERS";
    const ENV_HEADERS_SEPARATOR: &str = "HEADERS_SEPARATOR";

    static SERVICE_NAME: Lazy<String> = Lazy::new(|| {
        env::var(ENV_SERVICE_NAME).unwrap_or_else(|_| DEFAULT_SERVICE_NAME.to_string())
    });

    static COLLECTOR_URL: Lazy<String> =
        Lazy::new(|| env::var(ENV_COLLECTOR_URL).expect("collector url must be set"));

    static COLLECTOR_HEADERS: Lazy<String> =
        Lazy::new(|| env::var(ENV_COLLECTOR_HEADERS).unwrap_or_default());

    static HEADERS_SEPARATOR: Lazy<String> = Lazy::new(|| {
        env::var(ENV_HEADERS_SEPARATOR).unwrap_or(DEFAULT_HEADERS_SEPARATOR.to_string())
    });

    pub fn init_global_tracer() -> Result<(), SetGlobalDefaultError> {
        let metadata = MetadataMap::from_headers(HeaderMap::from_iter(
            COLLECTOR_HEADERS.split(&*HEADERS_SEPARATOR).map(|split| {
                let header: Vec<&str> = split.split(KEY_VALUE_SEPARATOR).collect();
                (
                    header[0].try_into().expect("header name must be valid"),
                    header[1].try_into().expect("header value must be valid"),
                )
            }),
        ));

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&*COLLECTOR_URL)
                    .with_metadata(metadata),
            )
            .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new(resource::SERVICE_NAME, SERVICE_NAME.clone()),
            ])))
            .install_batch(runtime::Tokio)
            .unwrap();

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let stdout_log = tracing_subscriber::fmt::layer().pretty();

        let subscriber = Registry::default()
            .with(telemetry)
            .with(stdout_log.with_filter(LevelFilter::INFO));

        tracing::subscriber::set_global_default(subscriber)
    }

    pub fn shutdown_global_tracer() {
        global::shutdown_tracer_provider()
    }
}
