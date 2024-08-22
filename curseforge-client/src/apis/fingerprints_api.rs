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

/// struct for passing parameters to the method [`FingerprintsApi::get_fingerprint_fuzzy_matches`]
#[derive(Clone, Debug)]
pub struct GetFingerprintFuzzyMatchesParams<'l1,> {
    pub get_fuzzy_matches_request_body: &'l1 GetFuzzyMatchesRequestBody,
}

/// struct for passing parameters to the method [`FingerprintsApi::get_fingerprint_fuzzy_matches_by_game`]
#[derive(Clone, Debug)]
pub struct GetFingerprintFuzzyMatchesByGameParams<'l2,> {
    /// The game id the find matches in
    pub game_id: u32,
    pub get_fuzzy_matches_request_body: &'l2 GetFuzzyMatchesRequestBody,
}

/// struct for passing parameters to the method [`FingerprintsApi::get_fingerprint_matches`]
#[derive(Clone, Debug)]
pub struct GetFingerprintMatchesParams<'l1,> {
    pub get_fingerprint_matches_request_body: &'l1 GetFingerprintMatchesRequestBody,
}

/// struct for passing parameters to the method [`FingerprintsApi::get_fingerprint_matches_by_game`]
#[derive(Clone, Debug)]
pub struct GetFingerprintMatchesByGameParams<'l2,> {
    /// The game id the find matches in
    pub game_id: u32,
    pub get_fingerprint_matches_request_body: &'l2 GetFingerprintMatchesRequestBody,
}


/// struct for typed errors of method [`FingerprintsApi::get_fingerprint_fuzzy_matches`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetFingerprintFuzzyMatchesError {
    #[error("Bad Request")]
    Status400(),
    #[error("Service Unavailable")]
    Status503(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`FingerprintsApi::get_fingerprint_fuzzy_matches_by_game`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetFingerprintFuzzyMatchesByGameError {
    #[error("Bad Request")]
    Status400(),
    #[error("Service Unavailable")]
    Status503(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`FingerprintsApi::get_fingerprint_matches`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetFingerprintMatchesError {
    #[error("Bad Request")]
    Status400(),
    #[error("Service Unavailable")]
    Status503(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`FingerprintsApi::get_fingerprint_matches_by_game`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetFingerprintMatchesByGameError {
    #[error("Bad Request")]
    Status400(),
    #[error("Service Unavailable")]
    Status503(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}

pub struct FingerprintsApi<'c>(pub(crate) &'c crate::ApiClient);
impl<'c> FingerprintsApi<'c> {
    /// Get mod files that match a list of fingerprints using fuzzy matching.
    pub async fn get_fingerprint_fuzzy_matches(&self, params: &GetFingerprintFuzzyMatchesParams<'_,>) -> Result<models::GetFingerprintFuzzyMatchesResponse> {
        // unwrap the parameters
        let GetFingerprintFuzzyMatchesParams { get_fuzzy_matches_request_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/v1/fingerprints/fuzzy"),
        )?;

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
        local_var_req_builder = local_var_req_builder.json(get_fuzzy_matches_request_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetFingerprintFuzzyMatchesError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    /// Get mod files that match a list of fingerprints using fuzzy matching.
    pub async fn get_fingerprint_fuzzy_matches_by_game(&self, params: &GetFingerprintFuzzyMatchesByGameParams<'_,>) -> Result<models::GetFingerprintFuzzyMatchesResponse> {
        // unwrap the parameters
        let GetFingerprintFuzzyMatchesByGameParams { game_id, get_fuzzy_matches_request_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/v1/fingerprints/fuzzy/{gameId}", gameId=game_id),
        )?;

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
        local_var_req_builder = local_var_req_builder.json(get_fuzzy_matches_request_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetFingerprintFuzzyMatchesByGameError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    /// Get mod files that match a list of fingerprints (murmur2 hashes with seed 1).
    pub async fn get_fingerprint_matches(&self, params: &GetFingerprintMatchesParams<'_,>) -> Result<models::GetFingerprintMatchesResponse> {
        // unwrap the parameters
        let GetFingerprintMatchesParams { get_fingerprint_matches_request_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/v1/fingerprints"),
        )?;

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
        local_var_req_builder = local_var_req_builder.json(get_fingerprint_matches_request_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetFingerprintMatchesError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    /// Get mod files that match a list of fingerprints.
    pub async fn get_fingerprint_matches_by_game(&self, params: &GetFingerprintMatchesByGameParams<'_,>) -> Result<models::GetFingerprintMatchesResponse> {
        // unwrap the parameters
        let GetFingerprintMatchesByGameParams { game_id, get_fingerprint_matches_request_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/v1/fingerprints/{gameId}", gameId=game_id),
        )?;

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
        local_var_req_builder = local_var_req_builder.json(get_fingerprint_matches_request_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetFingerprintMatchesByGameError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

}
