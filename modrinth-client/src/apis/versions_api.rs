/*
 * Labrinth
 *
 * This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:  - [cURL](https://curl.se/) (recommended, command-line) - [ReqBIN](https://reqbin.com/) (recommended, online) - [Postman](https://www.postman.com/downloads/) - [Insomnia](https://insomnia.rest/) - Your web browser, if you don't need to send headers or a request body  Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:  ```json {   \"about\": \"Welcome traveler!\",   \"documentation\": \"https://docs.modrinth.com\",   \"name\": \"modrinth-labrinth\",   \"version\": \"2.7.0\" } ```  If you got a response similar to the one above, you can use the Modrinth API! When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.  ## Authentication This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth). All tokens are tied to a Modrinth user and use the `Authorization` header of the request.  Example: ``` Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic ```  You do not need a token for most requests. Generally speaking, only the following types of requests require a token: - those which create data (such as version creation) - those which modify data (such as editing a project) - those which access private data (such as draft projects, notifications, emails, and payout data)  Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope. You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/pats.rs#L15). Making a request with an invalid scope will return a 401 error.  Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth. For example, deleting a user account can only be done through Modrinth's frontend.  ### OAuth2 Applications interacting with the authenticated API should create an OAuth2 application. You can do this in [the developer settings](https://modrinth.com/settings/applications).  Once you have created a client, use the following URL to have a user authorize your client: ``` https://modrinth.com/auth/authorize?client_id=<CLIENT_ID>&redirect_uri=<CALLBACK_URL>&scope=<SCOPE_ONE>+<SCOPE_TWO>+<SCOPE_THREE> ```  Then, use the following URL to get the token: ``` https://api.modrinth.com/_internal/oauth/token ```  This route will be changed in the future to move the `_internal` part to `v3`.  ### Personal access tokens Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).  ### GitHub tokens For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes. **We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.** GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.  ## Cross-Origin Resource Sharing This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/). This allows for cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.  ## Identifiers The majority of items you can interact with in the API have a unique eight-digit base62 ID. Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves. Version files use the sha1 or sha512 file hashes as identifiers.  Each project and user has a friendlier way of identifying them; slugs and usernames, respectively. While unique IDs are constant, slugs and usernames can change at any moment. If you want to store something in the long term, it is recommended to use the unique ID.  ## Ratelimits The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers. - `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute - `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window - `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets  Ratelimits are the same no matter whether you use a token or not. The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).  ## User Agents To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header. Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic. It is recommended, but not required, to include contact information in your user agent. This allows us to contact you if we would like a change in your application's behavior without having to block your traffic. - Bad: `User-Agent: okhttp/4.9.3` - Good: `User-Agent: project_name` - Better: `User-Agent: github_username/project_name/1.56.0` - Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`  ## Versioning Modrinth follows a simple pattern for its API versioning. In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.  When an API is no longer the current one, it will immediately be considered deprecated. No more support will be provided for API versions older than the current one. It will be kept for some time, but this amount of time is not certain.  We will exercise various tactics to get people to update their implementation of our API. One example is by adding something like `STOP USING THIS API` to various data returned by the API.  Once an API version is completely deprecated, it will permanently return a 410 error. Please ensure your application handles these 410 errors.  ### Migrations Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.  Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.  <details><summary>API v1 to API v2</summary>  These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.  - The search route has been moved from `/api/v1/mod` to `/v2/search` - New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery` - New search facet: `project_type` - Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch) - New search fields: `project_type`, `gallery`   - The gallery field is an array of URLs to images that are part of the project's gallery - The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page - Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.   - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth - In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft   - Similarly, for version creation, a version may not be uploaded without any files - Donation URLs have been enabled - New project status: `archived`. Projects with this status do not appear in search - Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached - Dependencies have been wiped and replaced with a new system - Notifications now have a `type` field, such as `project_update`  Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.  </details><details><summary>Minotaur v1 to Minotaur v2</summary>  Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.  First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.  To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.  Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.  Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.  You may now use the slug anywhere that a project ID was previously required.  </details>
 *
 * The version of the OpenAPI document: v2.7.0/15cf3fc
 * Contact: support@modrinth.com
 * Generated by: https://openapi-generator.tech
 */


use crate::{
    models::{self, *},
    ErrorResponse, Result,
};

/// struct for passing parameters to the method [`VersionsApi::add_files_to_version`]
#[derive(Clone, Debug)]
pub struct AddFilesToVersionParams<'l1,'l2,> {
    /// The ID of the version
    pub id: &'l1 str,
    pub data: Option<&'l2 serde_json::Value>,
}

/// struct for passing parameters to the method [`VersionsApi::create_version`]
#[derive(Clone, Debug)]
pub struct CreateVersionParams<'l1,> {
    pub data: &'l1 models::CreatableVersion,
}

/// struct for passing parameters to the method [`VersionsApi::delete_version`]
#[derive(Clone, Debug)]
pub struct DeleteVersionParams<'l1,> {
    /// The ID of the version
    pub id: &'l1 str,
}

/// struct for passing parameters to the method [`VersionsApi::get_project_versions`]
#[derive(Clone, Debug)]
pub struct GetProjectVersionsParams<'l1,'l2,'l3,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// The types of loaders to filter for
    pub loaders: Option<Vec<&'l2 str>>, // MANUAL CHANGE
    /// The game versions to filter for
    pub game_versions: Option<Vec<&'l3 str>>, // MANUAL CHANGE
    /// Allows to filter for featured or non-featured versions only
    pub featured: Option<bool>,
}

/// struct for passing parameters to the method [`VersionsApi::get_version`]
#[derive(Clone, Debug)]
pub struct GetVersionParams<'l1,> {
    /// The ID of the version
    pub id: &'l1 str,
}

/// struct for passing parameters to the method [`VersionsApi::get_version_from_id_or_number`]
#[derive(Clone, Debug)]
pub struct GetVersionFromIdOrNumberParams<'l1,'l2,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// The version ID or version number
    pub id: &'l2 str,
}

/// struct for passing parameters to the method [`VersionsApi::get_versions`]
#[derive(Clone, Debug)]
pub struct GetVersionsParams<'l1,> {
    /// The IDs of the versions
    pub ids: &'l1 [&'l1 str], // MANUAL CHANGE
}

/// struct for passing parameters to the method [`VersionsApi::modify_version`]
#[derive(Clone, Debug)]
pub struct ModifyVersionParams<'l1,'l2,> {
    /// The ID of the version
    pub id: &'l1 str,
    /// Modified version fields
    pub editable_version: Option<&'l2 EditableVersion>,
}

/// struct for passing parameters to the method [`VersionsApi::schedule_version`]
#[derive(Clone, Debug)]
pub struct ScheduleVersionParams<'l1,'l2,> {
    /// The ID of the version
    pub id: &'l1 str,
    /// Information about date and requested status
    pub schedule: Option<&'l2 Schedule>,
}


/// struct for typed errors of method [`VersionsApi::add_files_to_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum AddFilesToVersionError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::create_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum CreateVersionError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::delete_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteVersionError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::get_project_versions`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetProjectVersionsError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::get_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::get_version_from_id_or_number`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionFromIdOrNumberError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::get_versions`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetVersionsError {
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::modify_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ModifyVersionError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404(),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}
/// struct for typed errors of method [`VersionsApi::schedule_version`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ScheduleVersionError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    UnknownValue(serde_json::Value),
}

pub struct VersionsApi<'c>(pub(crate) &'c crate::ApiClient);
impl<'c> VersionsApi<'c> {
    /// Project files are attached. `.mrpack` and `.jar` files are accepted.
    pub async fn add_files_to_version(&self, params: &AddFilesToVersionParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let AddFilesToVersionParams { id, data, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/version/{id}/file", id=crate::urlencode(id)),
        )?;

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
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }
        let mut local_var_form = reqwest::multipart::Form::new();
        if let Some(ref data) = data {
            local_var_form = local_var_form.text("data", serde_json::to_string(data)?);
        }
        local_var_req_builder = local_var_req_builder.multipart(local_var_form);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            Ok(())
        } else {
            let local_var_entity = serde_json::from_str::<AddFilesToVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    /// This route creates a version on an existing project. There must be at least one file attached to each new version, unless the new version's status is `draft`. `.mrpack`, `.jar`, `.zip`, and `.litemod` files are accepted.  The request is a [multipart request](https://www.ietf.org/rfc/rfc2388.txt) with at least two form fields: one is `data`, which includes a JSON body with the version metadata as shown below, and at least one field containing an upload file.  You can name the file parts anything you would like, but you must list each of the parts' names in `file_parts`, and optionally, provide one to use as the primary file in `primary_file`.
    pub async fn create_version(&self, params: &CreateVersionParams<'_,>) -> Result<models::Version> {
        // unwrap the parameters
        let CreateVersionParams { data, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/version"),
        )?;

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
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }
        let mut local_var_form = reqwest::multipart::Form::new();
        local_var_form = local_var_form.text("data", serde_json::to_string(data)?);
        local_var_req_builder = local_var_req_builder.multipart(local_var_form);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<CreateVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn delete_version(&self, params: &DeleteVersionParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let DeleteVersionParams { id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format!("/version/{id}", id=crate::urlencode(id)),
        )?;

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
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            Ok(())
        } else {
            let local_var_entity = serde_json::from_str::<DeleteVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn get_project_versions(&self, params: &GetProjectVersionsParams<'_,'_,'_,>) -> Result<Vec<models::Version>> {
        // unwrap the parameters
        let GetProjectVersionsParams { mod_id, loaders, game_versions, featured, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format!("/project/{id_slug}/version", id_slug=crate::urlencode(mod_id)),
        )?;

        if let Some(ref loaders) = loaders {
            local_var_req_builder = local_var_req_builder.query(&[("loaders", serde_json::to_string(loaders)?)]); // MANUAL CHANGE
        }

        if let Some(ref game_versions) = game_versions {
            local_var_req_builder = local_var_req_builder.query(&[("game_versions", serde_json::to_string(game_versions)?)]); // MANUAL CHANGE
        }

        if let Some(ref featured) = featured {
            local_var_req_builder = local_var_req_builder.query(&[("featured", featured)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetProjectVersionsError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn get_version(&self, params: &GetVersionParams<'_,>) -> Result<models::Version> {
        // unwrap the parameters
        let GetVersionParams { id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format!("/version/{id}", id=crate::urlencode(id)),
        )?;

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    /// Please note that, if the version number provided matches multiple versions, only the **oldest matching version** will be returned.
    pub async fn get_version_from_id_or_number(&self, params: &GetVersionFromIdOrNumberParams<'_,'_,>) -> Result<models::Version> {
        // unwrap the parameters
        let GetVersionFromIdOrNumberParams { mod_id, id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format!("/project/{id_slug}/version/{id_number}", id_slug=crate::urlencode(mod_id), id_number=crate::urlencode(id)),
        )?;

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetVersionFromIdOrNumberError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn get_versions(&self, params: &GetVersionsParams<'_,>) -> Result<Vec<models::Version>> {
        // unwrap the parameters
        let GetVersionsParams { ids, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format!("/versions"),
        )?;

        local_var_req_builder = local_var_req_builder.query(&[("ids", serde_json::to_string(ids)?)]); // MANUAL CHANGE

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        } else {
            let local_var_entity = serde_json::from_str::<GetVersionsError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn modify_version(&self, params: &ModifyVersionParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ModifyVersionParams { id, editable_version, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            format!("/version/{id}", id=crate::urlencode(id)),
        )?;

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
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }
        local_var_req_builder = local_var_req_builder.json(editable_version);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            Ok(())
        } else {
            let local_var_entity = serde_json::from_str::<ModifyVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

    pub async fn schedule_version(&self, params: &ScheduleVersionParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ScheduleVersionParams { id, schedule, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format!("/version/{id}/schedule", id=crate::urlencode(id)),
        )?;

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
                local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, reqwest::header::HeaderValue::from_str(&cookies.join("; "))?);
            }
        }
        local_var_req_builder = local_var_req_builder.json(schedule);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
            Ok(())
        } else {
            let local_var_entity = serde_json::from_str::<ScheduleVersionError>(&local_var_content).map(|e| Box::new(e) as _).ok();
            let local_var_error = ErrorResponse { status: local_var_status, content: local_var_content, source: local_var_entity };
            Err(local_var_error.into())
        }
    }

}
