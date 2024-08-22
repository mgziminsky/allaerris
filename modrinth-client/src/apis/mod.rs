pub mod misc_api;
pub use misc_api::MiscApi;
pub mod notifications_api;
pub use notifications_api::NotificationsApi;
pub mod projects_api;
pub use projects_api::ProjectsApi;
pub mod tags_api;
pub use tags_api::TagsApi;
pub mod teams_api;
pub use teams_api::TeamsApi;
pub mod threads_api;
pub use threads_api::ThreadsApi;
pub mod users_api;
pub use users_api::UsersApi;
pub mod version_files_api;
pub use version_files_api::VersionFilesApi;
pub mod versions_api;
pub use versions_api::VersionsApi;

#[allow(dead_code)]
pub(crate) fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                },
                serde_json::Value::String(s) => params.push((format!("{}[{}]", prefix, key), s.clone())),
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}
