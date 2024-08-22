pub mod categories_api;
pub use categories_api::CategoriesApi;
pub mod files_api;
pub use files_api::FilesApi;
pub mod fingerprints_api;
pub use fingerprints_api::FingerprintsApi;
pub mod games_api;
pub use games_api::GamesApi;
pub mod minecraft_api;
pub use minecraft_api::MinecraftApi;
pub mod mods_api;
pub use mods_api::ModsApi;

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
