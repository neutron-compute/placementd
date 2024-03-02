///
/// The kube-provisioner worker simply looks for tasks to spawn into Kubernetes
///
use placementd::db::Task;
use sqlx::postgres::PgListener;
use tracing::log::*;
use uuid::Uuid;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Starting placementd kube-provisioner");

    let pool = placementd::db::bootstrap().await;
    let mut listener = PgListener::connect_with(&pool).await?;

    listener.listen("tasks-modified").await?;

    loop {
        let notification = listener.recv().await?;
        info!("notification: {notification:?}");
        let uuid = Uuid::try_parse(notification.payload())?;
        let task = Task::lookup(&uuid, &pool).await?;
        info!("task: {task:?}");

        /*
         * After a notified task has been executed, pick up another planned task to make sure
         * dropped tasks are executed
         */
    }
}
