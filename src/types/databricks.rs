///
/// Types representing Databricks API entities
///
use serde::Deserialize;
use std::collections::HashMap;

/// A struct representing the Databricks runs-submit API request
///
/// Not every value of the request is deserialized, only what is necessary for placementd to
/// create the task
///
/// [Create and trigger a one-time run](https://docs.databricks.com/api/workspace/jobs/submit)
#[derive(Clone, Debug, Default, Deserialize)]
pub struct RunsSubmitRequest {
    tasks: Vec<SubmitTask>,
    #[serde(rename = "run_name")]
    name: String,
    #[serde(rename = "timeout_seconds")]
    timeout: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SubmitTask {
    #[serde(rename = "task_key")]
    key: String,
    description: String,
    #[serde(rename = "timeout_seconds")]
    timeout: u64,
    #[serde(default = "Vec::new")]
    libraries: Vec<SubmitJar>,
    #[serde(rename = "spark_jar_task")]
    spark: Option<SparkTask>,
    #[serde(rename = "new_cluster", default = "SparkCluster::default")]
    cluster: SparkCluster,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SparkCluster {
    #[serde(rename = "spark_version")]
    version: String,
    #[serde(rename = "spark_conf")]
    conf: HashMap<String, serde_json::Value>,
    autoscale: ScaleConfiguration,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScaleConfiguration {
    #[serde(rename = "min_workers")]
    min: u64,
    #[serde(rename = "max_workers")]
    max: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SparkTask {
    #[serde(rename = "main_class_name")]
    main: String,
    parameters: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SubmitJar {
    jar: String,
}
