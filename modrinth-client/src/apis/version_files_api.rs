/*
 * Labrinth
 *
 * This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:  - [cURL](https://curl.se/) (recommended, command-line) - [ReqBIN](https://reqbin.com/) (recommended, online) - [Postman](https://www.postman.com/downloads/) - [Insomnia](https://insomnia.rest/) - Your web browser, if you don't need to send headers or a request body  Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:  ```json {   \"about\": \"Welcome traveler!\",   \"documentation\": \"https://docs.modrinth.com\",   \"name\": \"modrinth-labrinth\",   \"version\": \"2.7.0\" } ```  If you got a response similar to the one above, you can use the Modrinth API! When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.  ## Authentication This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth). All tokens are tied to a Modrinth user and use the `Authorization` header of the request.  Example: ``` Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic ```  You do not need a token for most requests. Generally speaking, only the following types of requests require a token: - those which create data (such as version creation) - those which modify data (such as editing a project) - those which access private data (such as draft projects, notifications, emails, and payout data)  Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope. You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/pats.rs#L15). Making a request with an invalid scope will return a 401 error.  Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth. For example, deleting a user account can only be done through Modrinth's frontend.  A detailed guide on OAuth has been published in [Modrinth's technical documentation](https://docs.modrinth.com/guide/oauth).  ### Personal access tokens Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).  ### GitHub tokens For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes. **We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.** GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.  ## Cross-Origin Resource Sharing This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/). This allows for cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.  ## Identifiers The majority of items you can interact with in the API have a unique eight-digit base62 ID. Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves. Version files use the sha1 or sha512 file hashes as identifiers.  Each project and user has a friendlier way of identifying them; slugs and usernames, respectively. While unique IDs are constant, slugs and usernames can change at any moment. If you want to store something in the long term, it is recommended to use the unique ID.  ## Ratelimits The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers. - `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute - `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window - `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets  Ratelimits are the same no matter whether you use a token or not. The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).  ## User Agents To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header. Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic. It is recommended, but not required, to include contact information in your user agent. This allows us to contact you if we would like a change in your application's behavior without having to block your traffic. - Bad: `User-Agent: okhttp/4.9.3` - Good: `User-Agent: project_name` - Better: `User-Agent: github_username/project_name/1.56.0` - Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`  ## Versioning Modrinth follows a simple pattern for its API versioning. In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.  When an API is no longer the current one, it will immediately be considered deprecated. No more support will be provided for API versions older than the current one. It will be kept for some time, but this amount of time is not certain.  We will exercise various tactics to get people to update their implementation of our API. One example is by adding something like `STOP USING THIS API` to various data returned by the API.  Once an API version is completely deprecated, it will permanently return a 410 error. Please ensure your application handles these 410 errors.  ### Migrations Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.  Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.  <details><summary>API v1 to API v2</summary>  These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.  - The search route has been moved from `/api/v1/mod` to `/v2/search` - New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery` - New search facet: `project_type` - Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch) - New search fields: `project_type`, `gallery`   - The gallery field is an array of URLs to images that are part of the project's gallery - The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page - Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.   - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth - In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft   - Similarly, for version creation, a version may not be uploaded without any files - Donation URLs have been enabled - New project status: `archived`. Projects with this status do not appear in search - Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached - Dependencies have been wiped and replaced with a new system - Notifications now have a `type` field, such as `project_update`  Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.  </details><details><summary>Minotaur v1 to Minotaur v2</summary>  Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.  First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.  To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.  Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.  Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.  You may now use the slug anywhere that a project ID was previously required.  </details>
 *
 * The version of the OpenAPI document: v2.7.0/366f528
 * Contact: support@modrinth.com
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`VersionFilesApi::delete_file_from_hash`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct DeleteFileFromHashParams<'l1,'l2,'l3,> {
    /// The hash of the file, considering its byte content, and encoded in hexadecimal
    pub hash: &'l1 str,
    /// The algorithm of the hash
    pub algorithm: &'l2 str,
    /// Version ID to delete the version from, if multiple files of the same hash exist
    pub version_id: Option<&'l3 str>,
}

/// struct for passing parameters to the method [`VersionFilesApi::get_latest_version_from_hash`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetLatestVersionFromHashParams<'l1,'l2,'l3,> {
    /// The hash of the file, considering its byte content, and encoded in hexadecimal
    pub hash: &'l1 str,
    /// The algorithm of the hash
    pub algorithm: &'l2 str,
    /// Parameters of the updated version requested
    pub get_latest_version_from_hash_body: Option<&'l3 GetLatestVersionFromHashBody>,
}

/// struct for passing parameters to the method [`VersionFilesApi::get_latest_versions_from_hashes`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct GetLatestVersionsFromHashesParams<'l1, 'l2> { // MANUAL: 'l2
    /// Parameters of the updated version requested
    pub get_latest_versions_from_hashes_body: Option<&'l1 GetLatestVersionsFromHashesBody<'l2>>, // MANUAL: 'l2
}

/// struct for passing parameters to the method [`VersionFilesApi::version_from_hash`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct VersionFromHashParams<'l1,'l2,> {
    /// The hash of the file, considering its byte content, and encoded in hexadecimal
    pub hash: &'l1 str,
    /// The algorithm of the hash
    pub algorithm: &'l2 str,
    /// Whether to return multiple results when looking for this hash
    pub multiple: Option<bool>,
}

/// struct for passing parameters to the method [`VersionFilesApi::versions_from_hashes`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bon", derive(::bon::Builder))]
pub struct VersionsFromHashesParams<'l1,> {
    /// Hashes and algorithm of the versions requested
    pub hash_list: Option<&'l1 HashList>,
}


/// struct for typed errors of method [`VersionFilesApi::delete_file_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteFileFromHashError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`VersionFilesApi::get_latest_version_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetLatestVersionFromHashError {
    #[error("Request was invalid, see given error")]
    Status400,
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`VersionFilesApi::get_latest_versions_from_hashes`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetLatestVersionsFromHashesError {
    #[error("Request was invalid, see given error")]
    Status400,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`VersionFilesApi::version_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum VersionFromHashError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`VersionFilesApi::versions_from_hashes`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum VersionsFromHashesError {
    #[error("Request was invalid, see given error")]
    Status400,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}

pub struct VersionFilesApi<'c>(pub(crate) &'c crate::ApiClient);
impl VersionFilesApi<'_> {
    pub async fn delete_file_from_hash(&self, params: &DeleteFileFromHashParams<'_,'_,'_,>) -> Result<()> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/version_file/{hash}"
            , hash=crate::urlencode(params.hash)
            )
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.token_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                req_builder = req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                req_builder = req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        req_builder = req_builder.query(&[("algorithm", &params.algorithm)]);
        if let Some(ref param_value) = params.version_id {
            req_builder = req_builder.query(&[("version_id", &param_value)]);
        }

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            Ok(())
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                401 => DeleteFileFromHashError::Status401(serde_json::from_str(&content)?),
                404 => DeleteFileFromHashError::Status404,
                _ => DeleteFileFromHashError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    pub async fn get_latest_version_from_hash(&self, params: &GetLatestVersionFromHashParams<'_,'_,'_,>) -> Result<models::Version> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::POST,
            format_args!(
            "/version_file/{hash}/update"
            , hash=crate::urlencode(params.hash)
            )
        );
        req_builder = req_builder.query(&[("algorithm", &params.algorithm)]);
        req_builder = req_builder.json(&params.get_latest_version_from_hash_body);

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                400 => GetLatestVersionFromHashError::Status400,
                404 => GetLatestVersionFromHashError::Status404,
                _ => GetLatestVersionFromHashError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    /// This is the same as [`/version_file/{hash}/update`](#operation/getLatestVersionFromHash) except it accepts multiple hashes.
    pub async fn get_latest_versions_from_hashes(&self, params: &GetLatestVersionsFromHashesParams<'_,'_>) -> Result<std::collections::HashMap<String, models::Version>> {  // MANUAL: Additional lifetime
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::POST,
            "/version_files/update"
        );
        req_builder = req_builder.json(&params.get_latest_versions_from_hashes_body);

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                400 => GetLatestVersionsFromHashesError::Status400,
                _ => GetLatestVersionsFromHashesError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    pub async fn version_from_hash(&self, params: &VersionFromHashParams<'_,'_,>) -> Result<models::Version> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/version_file/{hash}"
            , hash=crate::urlencode(params.hash)
            )
        );
        req_builder = req_builder.query(&[("algorithm", &params.algorithm)]);
        if let Some(ref param_value) = params.multiple {
            req_builder = req_builder.query(&[("multiple", &param_value)]);
        }

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                404 => VersionFromHashError::Status404,
                _ => VersionFromHashError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
    /// This is the same as [`/version_file/{hash}`](#operation/versionFromHash) except it accepts multiple hashes.
    pub async fn versions_from_hashes(&self, params: &VersionsFromHashesParams<'_,>) -> Result<std::collections::HashMap<String, models::Version>> {
        #[allow(unused_mut)]
        let mut req_builder = self.0.request(
            reqwest::Method::POST,
            "/version_files"
        );
        req_builder = req_builder.json(&params.hash_list);

        let resp = req_builder.send().await?;

        let status = resp.status();
        let content = resp.text().await?;

        if !status.is_client_error() && !status.is_server_error() {
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            #[allow(clippy::match_single_binding)]
            let error = match status.as_u16() {
                400 => VersionsFromHashesError::Status400,
                _ => VersionsFromHashesError::Unknown(serde_json::from_str(&content)?),
            };
            Err(ErrorResponse { status, content, source: Some(error.into()) }.into())
        }
    }
}
