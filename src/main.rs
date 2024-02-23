use std::env::*;
use tracing::log::*;

mod api;
mod dal;
mod types;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Starting placementd");

    let state = crate::types::ServerState::from_env().await;
    let mut app = tide::with_state(state.clone());

    app.at("v1").nest(api::v1::routes(state)?);
    let bind_to = var("BIND_TO").unwrap_or("0.0.0.0:8080".into());
    info!("Starting the HTTP handler on {bind_to}");
    app.listen(bind_to).await?;

    Ok(())
}

#[cfg(test)]
mod tests {}
