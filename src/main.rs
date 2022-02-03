#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;
use tonic::transport::Server;
use r2d2_redis::{r2d2, RedisConnectionManager};
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

use rauth::smtp::Smtp;
use rauth::metadata::{
    repository::PostgresMetadataRepository,
};

use rauth::secret::repository::PostgresSecretRepository;

use rauth::user::{
    grpc::{UserServer, UserImplementation},
    repository::PostgresUserRepository,
    application::UserApplication,
};

use rauth::session::{
    grpc::{SessionServer, SessionImplementation},
    repository::RedisSessionRepository,
    application::SessionApplication,
};

const DEFAULT_NETW: &str = "127.0.0.1";
const DEFAULT_TEMPLATES_PATH: &str = "/etc/rauth/mailer/templates/*.html";
const DEFAULT_EMAIL_ISSUER: &str = "rauth";

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_NET: &str = "SERVICE_NETW";
const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
const ENV_RSA_SECRET: &str = "RSA_SECRET";
const ENV_RSA_PUBLIC: &str = "RSA_PUBLIC";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_REDIS_DSN: &str = "REDIS_DSN";
const ENV_SESSION_TIMEOUT: &str = "SESSION_TIMEOUT";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";
const ENV_POSTGRES_POOL: &str = "POSTGRES_POOL";
const ENV_REDIS_POOL: &str = "REDIS_POOL";
const ENV_ALLOW_UNVERIFIED: &str = "ALLOW_UNVERIFIED";
const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_ISSUER: &str = "SMTP_ISSUER";
const ENV_SMTP_TEMPLATES: &str = "SMTP_TEMPLATES";
const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";

type PgPool = Pool<ConnectionManager<PgConnection>>;
type RdPool = r2d2::Pool<RedisConnectionManager> ;

lazy_static! {
    static ref SESSION_TIMEOUT: u64 = env::var(ENV_SESSION_TIMEOUT).expect("session's timeout must be set").parse().unwrap();
    static ref TOKEN_TIMEOUT: u64 = env::var(ENV_TOKEN_TIMEOUT).expect("token's timeout must be set").parse().unwrap();
    static ref RSA_SECRET: Vec<u8> = base64::decode(env::var(ENV_RSA_SECRET).expect("rsa secret must be set")).unwrap();
    static ref RSA_PUBLIC: Vec<u8> = base64::decode(env::var(ENV_RSA_PUBLIC).expect("rsa public key must be set")).unwrap();
    static ref JWT_SECRET: Vec<u8> = base64::decode(env::var(ENV_JWT_SECRET).expect("jwt secret must be set")).unwrap();
    static ref JWT_PUBLIC: Vec<u8> = base64::decode(env::var(ENV_JWT_PUBLIC).expect("jwt public key must be set")).unwrap();
    static ref JWT_HEADER: String = env::var(ENV_JWT_HEADER).expect("token's header must be set");
    static ref SMTP_TRANSPORT: String = env::var(ENV_SMTP_TRANSPORT).expect("smtp transport must be set");
    static ref SMTP_USERNAME: String = env::var(ENV_SMTP_USERNAME).expect("smtp username must be set");
    static ref SMTP_PASSWORD: String = env::var(ENV_SMTP_PASSWORD).expect("smtp password must be set");
    static ref SMTP_ORIGIN: String = env::var(ENV_SMTP_ORIGIN).expect("smpt origin must be set");
    static ref SMTP_ISSUER: String = env::var(ENV_SMTP_ISSUER).unwrap_or(DEFAULT_EMAIL_ISSUER.to_string());
    static ref SMTP_TEMPLATES: String = env::var(ENV_SMTP_TEMPLATES).unwrap_or(DEFAULT_TEMPLATES_PATH.to_string());
    
    static ref ALLOW_UNVERIFIED: bool = env::var(ENV_ALLOW_UNVERIFIED)
        .map(|allow| {
            info!("unverified actions allowness set to {}", allow);
            return allow.parse().unwrap();
        })
        .unwrap_or_default();


    static ref PG_POOL: PgPool = {
        let postgres_dsn = env::var(ENV_POSTGRES_DSN).expect("postgres url must be set");
        let postgres_pool = env::var(ENV_POSTGRES_POOL).expect("postgres pool size must be set").parse().unwrap();
        match PgPool::builder().max_size(postgres_pool).build(ConnectionManager::new(&postgres_dsn)) {
            Ok(pool) => {
                info!("connection with postgres cluster established");
                pool
            },
            Err(err) => {
                error!("{}", err);
                panic!("cannot establish connection with {}", postgres_dsn);
            }
        }
    };

    static ref RD_POOL: RdPool = {
        let redis_dsn: String = env::var(ENV_REDIS_DSN).expect("redis url must be set");
        let redis_pool = env::var(ENV_REDIS_POOL).expect("redis pool size must be set").parse().unwrap();
        let manager = RedisConnectionManager::new(redis_dsn.clone()).unwrap();
        match r2d2::Pool::builder().max_size(redis_pool).build(manager) {
            Ok(pool) => {
                info!("connection with redis cluster established");
                pool
            },
            Err(err) => {
                error!("{}", err);
                panic!("cannot establish connection with {}", redis_dsn);
            }
        }
    };
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let metadata_repo = Arc::new(PostgresMetadataRepository{
        pool: &PG_POOL,
    });

    let secret_repo = Arc::new(PostgresSecretRepository {
        pool: &PG_POOL,
        metadata_repo: metadata_repo.clone(),
    });

    let user_repo = Arc::new(PostgresUserRepository{
        pool: &PG_POOL,
        metadata_repo: metadata_repo.clone(),
    });

    let session_repo = Arc::new(RedisSessionRepository{
        pool: &RD_POOL,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
    });

    let credentials = if SMTP_USERNAME.len() > 0 && SMTP_PASSWORD.len() > 0 {
        Some((SMTP_USERNAME.to_string(), SMTP_PASSWORD.to_string()))
    } else {
        None
    };

    let mut mailer = Smtp::new(&SMTP_TEMPLATES, &SMTP_TRANSPORT, credentials)?;
    mailer.rsa_public = Some(&RSA_PUBLIC);
    mailer.issuer = &*SMTP_ISSUER;
    mailer.origin = &*SMTP_ORIGIN;

    let user_app = UserApplication{
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        mailer: Arc::new(mailer),
        timeout: *TOKEN_TIMEOUT,
    };

    let sess_app = SessionApplication{
        session_repo: session_repo.clone(),
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        timeout: *SESSION_TIMEOUT,
    };

    let user_server = UserImplementation{
        user_app: user_app,
        rsa_secret: &RSA_SECRET,
        rsa_public: &RSA_PUBLIC,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
        allow_unverified: *ALLOW_UNVERIFIED,
    };

    let sess_server = SessionImplementation {
        sess_app: sess_app,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
    };
 
    let addr = address.parse().unwrap();
    info!("server listening on {}", addr);
 
    Server::builder()
        .add_service(UserServer::new(user_server))
        .add_service(SessionServer::new(sess_server))
        .serve(addr)
        .await?;
 
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    if let Err(_) = dotenv::dotenv() {
        warn!("no dotenv file has been found");
    }

    let netw = match env::var(ENV_SERVICE_NET) {
        Err(_) => DEFAULT_NETW.to_string(),
        Ok(netw)  => netw,
    };

    let port = env::var(ENV_SERVICE_PORT)
        .expect("service port must be set");
    
    let addr = format!("{}:{}", netw, port);
    start_server(addr).await
}