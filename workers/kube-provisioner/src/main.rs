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

        match pool.begin().await {
            Ok(mut tx) => {
                if let Ok(task) = Task::lock(&mut tx, Some(&uuid), None).await {
                    info!("Found a task to work on: {task:?}");
                }

                if let Err(e) = tx.commit().await {
                    error!("Failed to commit transaction! {e:?}");
                }
            }
            Err(e) => error!("Failed to acquire a transaction: {e:?}"),
        }

        /*
         * Pick up a lingering task if one is availble, this adds some resiliency to ensure that
         * the tasks are not just worked up based on the NOTIFY
         */
        match pool.begin().await {
            Ok(mut tx) => {
                if let Ok(task) = Task::lock(&mut tx, None, None).await {
                    info!("Found a lingering task to work on: {task:?}");
                }

                if let Err(e) = tx.commit().await {
                    error!("Failed to commit transaction! {e:?}");
                }
            }
            Err(e) => error!("Failed to acquire a transaction: {e:?}"),
        }

        /*
         * After a notified task has been executed, pick up another planned task to make sure
         * dropped tasks are executed
         */
    }
}
