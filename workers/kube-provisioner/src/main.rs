///
/// The kube-provisioner worker simply looks for tasks to spawn into Kubernetes
///
use kube::api::DynamicObject;
use placementd::db::{Task, Transaction};
use serde::Deserialize;
use sqlx::postgres::PgListener;
use tracing::log::*;
use uuid::Uuid;

use std::fs::File;


use kube::{Client, api::{Api, ResourceExt, ListParams, PostParams}};
use k8s_openapi::api::core::v1::Pod;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(_) = dotenvy::dotenv() {
        //
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    // The resources vec contains all the Spark resources to launch in kubernetes
    let mut resources: Vec<DynamicObject> = vec![];

    let cwd = std::env::current_dir()?;
    info!("Starting in {cwd:?}");
    let bundled = cwd.join("config/bundled/spark.yml");
    if !bundled.exists() {
        panic!("The configuration file is not present! I can't do anything without it, help!");
    }
    info!("Loading configuration defaults from: {bundled:?}");
    for document in serde_yaml::Deserializer::from_reader(File::open(bundled)?) {
        resources.push(DynamicObject::deserialize(document)?);
    }

    let spark_overrides = cwd.join("spark.overrides.yml");
    if spark_overrides.exists() {
        info!("Loading configuration overrides from {spark_overrides:?}");
        for document in serde_yaml::Deserializer::from_reader(File::open(spark_overrides)?) {
            let mut value = DynamicObject::deserialize(document)?;
            if let Some(types) = &value.types {
                let kind = &types.kind;
                let name = value.metadata.name.unwrap_or("unknown".into());
                debug!("Looking to override a {kind} named {name}");
                resources = resources
                    .into_iter()
                    .map(|mut resource| {
                        if let Some(types) = &resource.types {
                            if kind == &types.kind && resource.metadata.name.as_ref() == Some(&name)
                            {
                                placementd::merge_json(&mut value.data, resource.data.clone());
                                debug!("Configurtation data merged into: {:?}", value.data);
                                resource.data = value.data.clone();
                            }
                        }
                        resource
                    })
                    .collect();
            } else {
                warn!("Override file does not contain enough oof a YAML fragment to understand: {value:?}");
            }
        }
    }
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
                    dispatch_task(&mut tx, task, &resources).await;
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
                    dispatch_task(&mut tx, task, &resources).await;
                }

                if let Err(e) = tx.commit().await {
                    error!("Failed to commit transaction! {e:?}");
                }
            }
            Err(e) => error!("Failed to acquire a transaction: {e:?}"),
        }
    }
}

///
/// The dispatch_task function is responsible for the primary work of this worker, ensuring that
/// the [Task] is properly scheduled and the database is updated.
async fn dispatch_task(_tx: &mut Transaction<'_>, task: Task, _resources: &Vec<DynamicObject>) {
    debug!("Dispatching task ident {:?}", task.ident);

     // Infer the runtime environment and try to create a Kubernetes Client
    if let Ok(client) = Client::try_default().await {
        // Read pods in the configured namespace into the typed interface from k8s-openapi
        let pods: Api<Pod> = Api::default_namespaced(client);
        if let Ok(pods) = pods.list(&ListParams::default()).await {
            for p in pods {
                println!("found pod {}", p.name_any());
            }
        }
    }
}
