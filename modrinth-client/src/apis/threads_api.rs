/*
 * Labrinth
 *
 * This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:  - [cURL](https://curl.se/) (recommended, command-line) - [ReqBIN](https://reqbin.com/) (recommended, online) - [Postman](https://www.postman.com/downloads/) - [Insomnia](https://insomnia.rest/) - Your web browser, if you don't need to send headers or a request body  Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:  ```json {   \"about\": \"Welcome traveler!\",   \"documentation\": \"https://docs.modrinth.com\",   \"name\": \"modrinth-labrinth\",   \"version\": \"2.7.0\" } ```  If you got a response similar to the one above, you can use the Modrinth API! When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.  ## Authentication This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth). All tokens are tied to a Modrinth user and use the `Authorization` header of the request.  Example: ``` Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic ```  You do not need a token for most requests. Generally speaking, only the following types of requests require a token: - those which create data (such as version creation) - those which modify data (such as editing a project) - those which access private data (such as draft projects, notifications, emails, and payout data)  Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope. You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/pats.rs#L15). Making a request with an invalid scope will return a 401 error.  Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth. For example, deleting a user account can only be done through Modrinth's frontend.  ### OAuth2 Applications interacting with the authenticated API should create an OAuth2 application. You can do this in [the developer settings](https://modrinth.com/settings/applications).  Once you have created a client, use the following URL to have a user authorize your client: ``` https://modrinth.com/auth/authorize?client_id=<CLIENT_ID>&redirect_uri=<CALLBACK_URL>&scope=<SCOPE_ONE>+<SCOPE_TWO>+<SCOPE_THREE> ```  Then, use the following URL to get the token: ``` https://api.modrinth.com/_internal/oauth/token ```  This route will be changed in the future to move the `_internal` part to `v3`.  ### Personal access tokens Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).  ### GitHub tokens For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes. **We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.** GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.  ## Cross-Origin Resource Sharing This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/). This allows for cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.  ## Identifiers The majority of items you can interact with in the API have a unique eight-digit base62 ID. Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves. Version files use the sha1 or sha512 file hashes as identifiers.  Each project and user has a friendlier way of identifying them; slugs and usernames, respectively. While unique IDs are constant, slugs and usernames can change at any moment. If you want to store something in the long term, it is recommended to use the unique ID.  ## Ratelimits The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers. - `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute - `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window - `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets  Ratelimits are the same no matter whether you use a token or not. The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).  ## User Agents To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header. Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic. It is recommended, but not required, to include contact information in your user agent. This allows us to contact you if we would like a change in your application's behavior without having to block your traffic. - Bad: `User-Agent: okhttp/4.9.3` - Good: `User-Agent: project_name` - Better: `User-Agent: github_username/project_name/1.56.0` - Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`  ## Versioning Modrinth follows a simple pattern for its API versioning. In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.  When an API is no longer the current one, it will immediately be considered deprecated. No more support will be provided for API versions older than the current one. It will be kept for some time, but this amount of time is not certain.  We will exercise various tactics to get people to update their implementation of our API. One example is by adding something like `STOP USING THIS API` to various data returned by the API.  Once an API version is completely deprecated, it will permanently return a 410 error. Please ensure your application handles these 410 errors.  ### Migrations Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.  Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.  <details><summary>API v1 to API v2</summary>  These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.  - The search route has been moved from `/api/v1/mod` to `/v2/search` - New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery` - New search facet: `project_type` - Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch) - New search fields: `project_type`, `gallery`   - The gallery field is an array of URLs to images that are part of the project's gallery - The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page - Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.   - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth - In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft   - Similarly, for version creation, a version may not be uploaded without any files - Donation URLs have been enabled - New project status: `archived`. Projects with this status do not appear in search - Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached - Dependencies have been wiped and replaced with a new system - Notifications now have a `type` field, such as `project_update`  Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.  </details><details><summary>Minotaur v1 to Minotaur v2</summary>  Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.  First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.  To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.  Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.  Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.  You may now use the slug anywhere that a project ID was previously required.  </details> 
 *
 * The version of the OpenAPI document: v2.7.0/15cf3fc
 * Contact: support@modrinth.com
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`ThreadsApi::delete_thread_message`]
#[derive(Clone, Debug)]
pub struct DeleteThreadMessageParams<'l1,> {
    /// The ID of the message
    pub id: &'l1 str,
}

/// struct for passing parameters to the method [`ThreadsApi::get_open_reports`]
#[derive(Clone, Debug)]
pub struct GetOpenReportsParams<> {
    pub count: Option<u32>,
}

/// struct for passing parameters to the method [`ThreadsApi::get_report`]
#[derive(Clone, Debug)]
pub struct GetReportParams<'l1,> {
    /// The ID of the report
    pub id: &'l1 str,
}

/// struct for passing parameters to the method [`ThreadsApi::get_reports`]
#[derive(Clone, Debug)]
pub struct GetReportsParams<'l1,> {
    /// The IDs of the reports
    pub ids: &'l1 str,
}

/// struct for passing parameters to the method [`ThreadsApi::get_thread`]
#[derive(Clone, Debug)]
pub struct GetThreadParams<'l1,> {
    /// The ID of the thread
    pub id: &'l1 str,
}

/// struct for passing parameters to the method [`ThreadsApi::get_threads`]
#[derive(Clone, Debug)]
pub struct GetThreadsParams<'l1,> {
    /// The IDs of the threads
    pub ids: &'l1 str,
}

/// struct for passing parameters to the method [`ThreadsApi::modify_report`]
#[derive(Clone, Debug)]
pub struct ModifyReportParams<'l1,'l2,> {
    /// The ID of the report
    pub id: &'l1 str,
    /// What to modify about the report
    pub modify_report_request: Option<&'l2 ModifyReportRequest>,
}

/// struct for passing parameters to the method [`ThreadsApi::send_thread_message`]
#[derive(Clone, Debug)]
pub struct SendThreadMessageParams<'l1,'l2,> {
    /// The ID of the thread
    pub id: &'l1 str,
    /// The message to be sent. Note that you only need the fields applicable for the `text` type.
    pub thread_message_body: Option<&'l2 ThreadMessageBody>,
}

/// struct for passing parameters to the method [`ThreadsApi::submit_report`]
#[derive(Clone, Debug)]
pub struct SubmitReportParams<'l1,> {
    /// The report to be sent
    pub creatable_report: Option<&'l1 CreatableReport>,
}


/// struct for typed errors of method [`ThreadsApi::delete_thread_message`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteThreadMessageError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::get_open_reports`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetOpenReportsError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::get_report`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetReportError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::get_reports`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetReportsError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::get_thread`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetThreadError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::get_threads`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetThreadsError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::modify_report`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ModifyReportError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::send_thread_message`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum SendThreadMessageError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ThreadsApi::submit_report`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum SubmitReportError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}

pub struct ThreadsApi<'c>(pub(crate) &'c crate::ApiClient);
impl<'c> ThreadsApi<'c> {
    pub async fn delete_thread_message(&self, params: &DeleteThreadMessageParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let DeleteThreadMessageParams { id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/message/{id}"
            , id=crate::urlencode(id)
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
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
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
                401 => DeleteThreadMessageError::Status401(serde_json::from_str(&local_var_content)?),
                404 => DeleteThreadMessageError::Status404,
                _ => DeleteThreadMessageError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn get_open_reports(&self, params: &GetOpenReportsParams<>) -> Result<Vec<models::Report>> {
        // unwrap the parameters
        let GetOpenReportsParams { count, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/report"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.token_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        if let Some(ref count) = count {
            local_var_req_builder = local_var_req_builder.query(&[("count", count)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                401 => GetOpenReportsError::Status401(serde_json::from_str(&local_var_content)?),
                404 => GetOpenReportsError::Status404,
                _ => GetOpenReportsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_report(&self, params: &GetReportParams<'_,>) -> Result<models::Report> {
        // unwrap the parameters
        let GetReportParams { id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/report/{id}"
            , id=crate::urlencode(id)
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
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
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
                401 => GetReportError::Status401(serde_json::from_str(&local_var_content)?),
                404 => GetReportError::Status404,
                _ => GetReportError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_reports(&self, params: &GetReportsParams<'_,>) -> Result<Vec<models::Report>> {
        // unwrap the parameters
        let GetReportsParams { ids, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/reports"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.token_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        local_var_req_builder = local_var_req_builder.query(&[("ids", ids)]);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                401 => GetReportsError::Status401(serde_json::from_str(&local_var_content)?),
                404 => GetReportsError::Status404,
                _ => GetReportsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_thread(&self, params: &GetThreadParams<'_,>) -> Result<models::Thread> {
        // unwrap the parameters
        let GetThreadParams { id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/thread/{id}"
            , id=crate::urlencode(id)
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
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
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
                404 => GetThreadError::Status404,
                _ => GetThreadError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_threads(&self, params: &GetThreadsParams<'_,>) -> Result<Vec<models::Thread>> {
        // unwrap the parameters
        let GetThreadsParams { ids, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/threads"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.token_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }

        local_var_req_builder = local_var_req_builder.query(&[("ids", ids)]);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetThreadsError::Status404,
                _ => GetThreadsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn modify_report(&self, params: &ModifyReportParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ModifyReportParams { id, modify_report_request, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            format_args!(
            "/report/{id}"
            , id=crate::urlencode(id)
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
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }
        local_var_req_builder = local_var_req_builder.json(modify_report_request);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => ModifyReportError::Status400(serde_json::from_str(&local_var_content)?),
                401 => ModifyReportError::Status401(serde_json::from_str(&local_var_content)?),
                404 => ModifyReportError::Status404,
                _ => ModifyReportError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn send_thread_message(&self, params: &SendThreadMessageParams<'_,'_,>) -> Result<models::Thread> {
        // unwrap the parameters
        let SendThreadMessageParams { id, thread_message_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format_args!(
            "/thread/{id}"
            , id=crate::urlencode(id)
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
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }
        local_var_req_builder = local_var_req_builder.json(thread_message_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => SendThreadMessageError::Status400(serde_json::from_str(&local_var_content)?),
                404 => SendThreadMessageError::Status404,
                _ => SendThreadMessageError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    /// Bring a project, user, or version to the attention of the moderators by reporting it.
    pub async fn submit_report(&self, params: &SubmitReportParams<'_,>) -> Result<models::Report> {
        // unwrap the parameters
        let SubmitReportParams { creatable_report, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            "/report"
        );

        // Auth
        #[allow(unused_mut)]
        {
            let auth = &self.0.auth;
            let mut cookies = Vec::<String>::new();
            if let Some(val) = &auth.token_auth {
                let mut val = reqwest::header::HeaderValue::from_str(val)?;
                val.set_sensitive(true);
                local_var_req_builder = local_var_req_builder.header("Authorization", val);
            }
            if !cookies.is_empty() {
                local_var_req_builder = local_var_req_builder.header(
                    reqwest::header::COOKIE,
                    reqwest::header::HeaderValue::from_str(&cookies.join("; "))?
                );
            }
        }
        local_var_req_builder = local_var_req_builder.json(creatable_report);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => SubmitReportError::Status400(serde_json::from_str(&local_var_content)?),
                401 => SubmitReportError::Status401(serde_json::from_str(&local_var_content)?),
                _ => SubmitReportError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

}
