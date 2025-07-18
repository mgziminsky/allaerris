/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.250410
 * 
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`MinecraftApi::get_minecraft_mod_loaders`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetMinecraftModLoadersParams<'l1,> {
    pub version: Option<&'l1 str>,
    pub include_all: Option<bool>,
}

/// struct for passing parameters to the method [`MinecraftApi::get_minecraft_versions`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetMinecraftVersionsParams<> {
    pub sort_descending: Option<bool>,
}

/// struct for passing parameters to the method [`MinecraftApi::get_specific_minecraft_mod_loader`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetSpecificMinecraftModLoaderParams<'l1,> {
    pub mod_loader_name: &'l1 str,
}

/// struct for passing parameters to the method [`MinecraftApi::get_specific_minecraft_version`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetSpecificMinecraftVersionParams<'l1,> {
    pub game_version_string: &'l1 str,
}


/// struct for typed errors of method [`MinecraftApi::get_minecraft_mod_loaders`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetMinecraftModLoadersError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`MinecraftApi::get_minecraft_versions`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetMinecraftVersionsError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`MinecraftApi::get_specific_minecraft_mod_loader`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetSpecificMinecraftModLoaderError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`MinecraftApi::get_specific_minecraft_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetSpecificMinecraftVersionError {
    #[error("Not Found")]
    Status404,
    #[error("Internal Server Error")]
    Status500,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}

pub struct MinecraftApi<'c>(pub(crate) &'c crate::ApiClient);
impl MinecraftApi<'_> {
    /// Get Minecraft ModLoaders
    pub async fn get_minecraft_mod_loaders(&self, params: &GetMinecraftModLoadersParams<'_,>) -> Result<models::ApiResponseOfListOfMinecraftModLoaderIndex> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::GET,
            "/v1/minecraft/modloader"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.api_key_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                req_builder = req_builder.header("x-api-key", val);
            }
            if !cookies.is_empty() {
                req_builder = req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        if let Some(ref param_value) = params.version {
            req_builder = req_builder.query(&[("version", &param_value)]);
        }
        if let Some(ref param_value) = params.include_all {
            req_builder = req_builder.query(&[("includeAll", &param_value)]);
        }

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                404 => GetMinecraftModLoadersError::Status404,
                500 => GetMinecraftModLoadersError::Status500,
                _ => GetMinecraftModLoadersError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    /// Get Minecraft Versions
    pub async fn get_minecraft_versions(&self, params: &GetMinecraftVersionsParams<>) -> Result<models::ApiResponseOfListOfMinecraftGameVersion> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::GET,
            "/v1/minecraft/version"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.api_key_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                req_builder = req_builder.header("x-api-key", val);
            }
            if !cookies.is_empty() {
                req_builder = req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        if let Some(ref param_value) = params.sort_descending {
            req_builder = req_builder.query(&[("sortDescending", &param_value)]);
        }

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                404 => GetMinecraftVersionsError::Status404,
                500 => GetMinecraftVersionsError::Status500,
                _ => GetMinecraftVersionsError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    /// Get Specific Minecraft ModLoader
    pub async fn get_specific_minecraft_mod_loader(&self, params: &GetSpecificMinecraftModLoaderParams<'_,>) -> Result<models::ApiResponseOfMinecraftModLoaderVersion> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v1/minecraft/modloader/{modLoaderName}"
            , modLoaderName=crate::urlencode(params.mod_loader_name)
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
                req_builder = req_builder.header("x-api-key", val);
            }
            if !cookies.is_empty() {
                req_builder = req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }


        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                404 => GetSpecificMinecraftModLoaderError::Status404,
                500 => GetSpecificMinecraftModLoaderError::Status500,
                _ => GetSpecificMinecraftModLoaderError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    /// Get Specific Minecraft Version
    pub async fn get_specific_minecraft_version(&self, params: &GetSpecificMinecraftVersionParams<'_,>) -> Result<models::ApiResponseOfMinecraftGameVersion> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/v1/minecraft/version/{gameVersionString}"
            , gameVersionString=crate::urlencode(params.game_version_string)
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
                req_builder = req_builder.header("x-api-key", val);
            }
            if !cookies.is_empty() {
                req_builder = req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }


        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                404 => GetSpecificMinecraftVersionError::Status404,
                500 => GetSpecificMinecraftVersionError::Status500,
                _ => GetSpecificMinecraftVersionError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
}
