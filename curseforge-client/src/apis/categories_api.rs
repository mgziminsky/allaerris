/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`CategoriesApi::get_categories`]
#[derive(Clone, Debug)]
pub struct GetCategoriesParams<> {
    /// A game unique id
    pub game_id: u64,
    /// A class unique id
    pub class_id: Option<u64>,
    /// A flag used to only return classes
    pub classes_only: Option<bool>,
}


/// struct for typed errors of method [`CategoriesApi::get_categories`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetCategoriesError {
    #[error("Not Found")]
    Status404(),
    #[error("Internal Server Error")]
    Status500(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}

pub struct CategoriesApi<'c>(pub(crate) &'c crate::ApiClient);
impl<'c> CategoriesApi<'c> {
    /// Get all available classes and categories of the specified game. Specify a game id for a list of all game categories, or a class id for a list of categories under that class.
    pub async fn get_categories(&self, params: &GetCategoriesParams<>) -> Result<models::GetCategoriesResponse> {
        // unwrap the parameters
        let GetCategoriesParams { game_id, class_id, classes_only, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/v1/categories"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.api_key_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                local_var_req_builder = local_var_req_builder.header("x-api-key", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }

        local_var_req_builder = local_var_req_builder.query(&[("gameId", game_id)]);

        if let Some(ref class_id) = class_id {
            local_var_req_builder = local_var_req_builder.query(&[("classId", class_id)]);
        }

        if let Some(ref classes_only) = classes_only {
            local_var_req_builder = local_var_req_builder.query(&[("classesOnly", classes_only)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetCategoriesError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

}
