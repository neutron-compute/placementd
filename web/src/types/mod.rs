///
///  The types module contains a lot of the common types used for placementd
///
pub mod databricks;
pub mod responses;

///
/// State object that can be passed into request handlers, etc
#[derive(Clone, Debug)]
pub struct ServerState {
    pub pool: sqlx::PgPool,
}

impl ServerState {
    pub async fn from_env() -> Self {
        let pool = placementd::db::bootstrap().await;
        Self { pool }
    }
}

/// Type alias to make all uses of [tide::Server] consistent
pub type Server = tide::Server<ServerState>;
/// Type alias to make uses of [tide::Request] consistent
pub type Request = tide::Request<ServerState>;
