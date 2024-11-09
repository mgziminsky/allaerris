/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.240719
 * 
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`GamesApi::get_game`]
#[derive(Clone, Debug)]
pub struct GetGameParams<> {
    /// A game unique id
    pub game_id: u64,
}

/// struct for passing parameters to the method [`GamesApi::get_games`]
#[derive(Clone, Debug)]
pub struct GetGamesParams<> {
    /// A zero based index of the first item to include in the response, the limit is: (index + pageSize <= 10,000).
    pub index: Option<u32>,
    /// The number of items to include in the response, the default/maximum value is 50.
    pub page_size: Option<u32>,
}

/// struct for passing parameters to the method [`GamesApi::get_version_types`]
#[derive(Clone, Debug)]
pub struct GetVersionTypesParams<> {
    /// A game unique id
    pub game_id: u64,
}

/// struct for passing parameters to the method [`GamesApi::get_versions`]
#[derive(Clone, Debug)]
pub struct GetVersionsParams<> {
    /// A game unique id
    pub game_id: u64,
}

/// struct for passing parameters to the method [`GamesApi::get_versions_v2`]
#[derive(Clone, Debug)]
pub struct GetVersionsV2Params<> {
    /// A game unique id
    pub game_id: u64,
}


/// struct for typed errors of method [`GamesApi::get_game`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetGameError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`GamesApi::get_games`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetGamesError {
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`GamesApi::get_version_types`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionTypesError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`GamesApi::get_versions`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionsError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`GamesApi::get_versions_v2`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionsV2Error {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}

pub struct GamesApi<'c>(pub(crate) &'c crate::ApiClient);
impl GamesApi<'_> {
    /// Get a single game. A private game is only accessible by its respective API key.
    pub async fn get_game(&self, params: &GetGameParams<>) -> Result<models::GetGameResponse> {
        // unwrap the parameters
        let GetGameParams { game_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v1/games/{gameId}"
            , gameId=game_id
            )
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
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetGameError::Status404,
                500 => GetGameError::Status500,
                _ => GetGameError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    /// Get all games that are available to the provided API key.
    pub async fn get_games(&self, params: &GetGamesParams<>) -> Result<models::GetGamesResponse> {
        // unwrap the parameters
        let GetGamesParams { index, page_size, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/v1/games"
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
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        if let Some(ref index) = index {
            local_var_req_builder = local_var_req_builder.query(&[("index", index)]);
        }

        if let Some(ref page_size) = page_size {
            local_var_req_builder = local_var_req_builder.query(&[("pageSize", page_size)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                500 => GetGamesError::Status500,
                _ => GetGamesError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    /// Get all available version types of the specified game. A private game is only accessible to its respective API key. Currently, when creating games via the CurseForge Core Console, you are limited to a single game version type. This means that this endpoint is probably not useful in most cases and is relevant mostly when handling existing games that have multiple game versions such as World of Warcraft and Minecraft (e.g. 517 for wow_retail). 
    pub async fn get_version_types(&self, params: &GetVersionTypesParams<>) -> Result<models::GetVersionTypesResponse> {
        // unwrap the parameters
        let GetVersionTypesParams { game_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v1/games/{gameId}/version-types"
            , gameId=game_id
            )
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
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetVersionTypesError::Status404,
                500 => GetVersionTypesError::Status500,
                _ => GetVersionTypesError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    /// Get all available versions for each known version type of the specified game. A private game is only accessible to its respective API key.
    pub async fn get_versions(&self, params: &GetVersionsParams<>) -> Result<models::GetVersionsResponse> {
        // unwrap the parameters
        let GetVersionsParams { game_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v1/games/{gameId}/versions"
            , gameId=game_id
            )
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
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetVersionsError::Status404,
                500 => GetVersionsError::Status500,
                _ => GetVersionsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    /// Get all available versions for each known version type of the specified game. A private game is only accessible to its respective API key.
    pub async fn get_versions_v2(&self, params: &GetVersionsV2Params<>) -> Result<models::GetVersionsV2Response> {
        // unwrap the parameters
        let GetVersionsV2Params { game_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v2/games/{gameId}/versions"
            , gameId=game_id
            )
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
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetVersionsV2Error::Status404,
                500 => GetVersionsV2Error::Status500,
                _ => GetVersionsV2Error::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

}
