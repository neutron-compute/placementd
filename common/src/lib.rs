///
/// Common functionality for all the components of placementd
pub mod db;

/// Merge two values
///
/// <https://stackoverflow.com/a/67743348>
pub fn merge_json(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                if v.is_array() && a.contains_key(&k) && a[&k].is_array() {
                    let mut _b = a.get(&k).unwrap().as_array().unwrap().to_owned();
                    _b.append(&mut v.as_array().unwrap().to_owned());
                    a[&k] = serde_json::Value::from(_b);
                    continue;
                }
                if !a.contains_key(&k) {
                    a.insert(k.to_owned(), v.to_owned());
                } else {
                    merge_json(&mut a[&k], v);
                }
            }
        }
        (a, b) => *a = b,
    }
}

/*
pub mod config {
    use serde::Deserialize;
    use url::Url;

    pub fn load<'life, T>(url: &Url) -> T where T: Deserialize<'life> {
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_config() {
        }
    }
}
*/
