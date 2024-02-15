use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{
        Api, DynamicObject, GroupVersionKind, ListParams, Patch, PatchParams, PostParams,
        ResourceExt,
    },
    Client, Discovery,
};
use std::env::*;
use std::fs::File;
use tracing::log::*;

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
    let mut app = tide::new();
    //app.at("static").serve_dir("www/static")?;

    /*
    let conf_dir = std::fs::canonicalize(var("CONF_DIR").unwrap_or("conf".into()))
        .expect("Failed to canonicalize CONF_DIR");
    info!("Using the configuration directory: {conf_dir:?}");

    // Infer the runtime environment and try to create a Kubernetes Client
    let client = Client::try_default().await?;
    let discovery = Discovery::new(client.clone()).run().await?;
    debug!("Kubernetes API client initialized");

    // Read pods in the configured namespace into the typed interface from k8s-openapi
    let pods: Api<Pod> = Api::default_namespaced(client.clone());
    for p in pods.list(&ListParams::default()).await? {
        println!("found pod {}", p.name_any());
    }

    let o: DynamicObject = serde_yaml::from_reader(File::open("conf/defaults/spark.manager.yml")?)?;
    println!("o: {o:?}");
    let gvk = GroupVersionKind::try_from(o.clone().types.unwrap())?;
    if let Some((api_resource, _caps)) = discovery.resolve_gvk(&gvk) {
        let api: Api<DynamicObject> = Api::default_namespaced_with(client.clone(), &api_resource);
        println!("api: {api:?}");
        let params = PostParams {
            dry_run: true,
            field_manager: None,
        };
        //let _r = api.patch("foo", &ssapply, &Patch::Apply(o)).await?;
        let _r = api.create(&params, &o).await?;
    }
    */

    let bind_to = var("BIND_TO").unwrap_or("0.0.0.0:8080".into());
    info!("Starting the HTTP handler on {bind_to}");
    app.listen(bind_to).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_yaml() {
        use serde::Deserialize;
        use std::fs::File;
        let file = File::open("./contrib/app.yml").expect("Failed to open");

        for document in serde_yaml::Deserializer::from_reader(file) {
            let value = serde_yaml::Value::deserialize(document).expect("Failed to deserialize");
            println!("{:?}", value);
        }
        assert!(false);
    }
}
