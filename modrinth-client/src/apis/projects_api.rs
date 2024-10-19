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

/// struct for passing parameters to the method [`ProjectsApi::add_gallery_image`]
#[derive(Clone, Debug)]
pub struct AddGalleryImageParams<'l1,'l2,'l4,'l5,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// Image extension
    pub ext: &'l2 str,
    /// Whether an image is featured
    pub featured: bool,
    /// Title of the image
    pub title: Option<&'l4 str>,
    /// Description of the image
    pub description: Option<&'l5 str>,
    /// Ordering of the image
    pub ordering: Option<u32>,
    pub body: Option<std::path::PathBuf>,
}

/// struct for passing parameters to the method [`ProjectsApi::change_project_icon`]
#[derive(Clone, Debug)]
pub struct ChangeProjectIconParams<'l1,'l2,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// Image extension
    pub ext: &'l2 str,
    pub body: Option<std::path::PathBuf>,
}

/// struct for passing parameters to the method [`ProjectsApi::check_project_validity`]
#[derive(Clone, Debug)]
pub struct CheckProjectValidityParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::create_project`]
#[derive(Clone, Debug)]
pub struct CreateProjectParams<'l1,> {
    pub data: &'l1 models::CreatableProject,
    /// Project icon file
    pub icon: Option<std::path::PathBuf>,
}

/// struct for passing parameters to the method [`ProjectsApi::delete_gallery_image`]
#[derive(Clone, Debug)]
pub struct DeleteGalleryImageParams<'l1,'l2,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// URL link of the image to delete
    pub url: &'l2 str,
}

/// struct for passing parameters to the method [`ProjectsApi::delete_project`]
#[derive(Clone, Debug)]
pub struct DeleteProjectParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::delete_project_icon`]
#[derive(Clone, Debug)]
pub struct DeleteProjectIconParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::follow_project`]
#[derive(Clone, Debug)]
pub struct FollowProjectParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::get_dependencies`]
#[derive(Clone, Debug)]
pub struct GetDependenciesParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::get_project`]
#[derive(Clone, Debug)]
pub struct GetProjectParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}

/// struct for passing parameters to the method [`ProjectsApi::get_projects`]
#[derive(Clone, Debug)]
pub struct GetProjectsParams<'l1,> {
    /// The IDs and/or slugs of the projects
    pub ids: &'l1 [&'l1 str], // MANUAL CHANGE
}

/// struct for passing parameters to the method [`ProjectsApi::modify_gallery_image`]
#[derive(Clone, Debug)]
pub struct ModifyGalleryImageParams<'l1,'l2,'l4,'l5,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// URL link of the image to modify
    pub url: &'l2 str,
    /// Whether the image is featured
    pub featured: Option<bool>,
    /// New title of the image
    pub title: Option<&'l4 str>,
    /// New description of the image
    pub description: Option<&'l5 str>,
    /// New ordering of the image
    pub ordering: Option<u32>,
}

/// struct for passing parameters to the method [`ProjectsApi::modify_project`]
#[derive(Clone, Debug)]
pub struct ModifyProjectParams<'l1,'l2,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// Modified project fields
    pub editable_project: Option<&'l2 EditableProject>,
}

/// struct for passing parameters to the method [`ProjectsApi::patch_projects`]
#[derive(Clone, Debug)]
pub struct PatchProjectsParams<'l1,'l2,> {
    /// The IDs and/or slugs of the projects
    pub ids: &'l1 [&'l1 str], // MANUAL CHANGE
    /// Fields to edit on all projects specified
    pub patch_projects_body: Option<&'l2 PatchProjectsBody>,
}

/// struct for passing parameters to the method [`ProjectsApi::random_projects`]
#[derive(Clone, Debug)]
pub struct RandomProjectsParams<> {
    /// The number of random projects to return
    pub count: u8,
}

/// struct for passing parameters to the method [`ProjectsApi::schedule_project`]
#[derive(Clone, Debug)]
pub struct ScheduleProjectParams<'l1,'l2,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
    /// Information about date and requested status
    pub schedule: Option<&'l2 Schedule>,
}

/// struct for passing parameters to the method [`ProjectsApi::search_projects`]
#[derive(Clone, Debug)]
pub struct SearchProjectsParams<'l1,'l2,'l3,> {
    /// The query to search for
    pub query: Option<&'l1 str>,
    /// Facets are an essential concept for understanding how to filter out results.  These are the most commonly used facet types: - `project_type` - `categories` (loaders are lumped in with categories in search) - `versions` - `client_side` - `server_side` - `open_source`  Several others are also available for use, though these should not be used outside very specific use cases. - `title` - `author` - `follows` - `project_id` - `license` - `downloads` - `color` - `created_timestamp` - `modified_timestamp`  In order to then use these facets, you need a value to filter by, as well as an operation to perform on this value. The most common operation is `:` (same as `=`), though you can also use `!=`, `>=`, `>`, `<=`, and `<`. Join together the type, operation, and value, and you've got your string. ``` {type} {operation} {value} ```  Examples: ``` categories = adventure versions != 1.20.1 downloads <= 100 ```  You then join these strings together in arrays to signal `AND` and `OR` operators.  ##### OR All elements in a single array are considered to be joined by OR statements. For example, the search `[[\"versions:1.16.5\", \"versions:1.17.1\"]]` translates to `Projects that support 1.16.5 OR 1.17.1`.  ##### AND Separate arrays are considered to be joined by AND statements. For example, the search `[[\"versions:1.16.5\"], [\"project_type:modpack\"]]` translates to `Projects that support 1.16.5 AND are modpacks`.
    pub facets: Option<&'l2 str>,
    /// The sorting method used for sorting search results
    pub index: Option<&'l3 str>,
    /// The offset into the search. Skips this number of results
    pub offset: Option<u32>,
    /// The number of results returned by the search
    pub limit: Option<u8>,
}

/// struct for passing parameters to the method [`ProjectsApi::unfollow_project`]
#[derive(Clone, Debug)]
pub struct UnfollowProjectParams<'l1,> {
    /// The ID or slug of the project
    pub mod_id: &'l1 str,
}


/// struct for typed errors of method [`ProjectsApi::add_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum AddGalleryImageError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::change_project_icon`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ChangeProjectIconError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::check_project_validity`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum CheckProjectValidityError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::create_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum CreateProjectError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::delete_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteGalleryImageError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::delete_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteProjectError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::delete_project_icon`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum DeleteProjectIconError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::follow_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum FollowProjectError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::get_dependencies`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetDependenciesError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::get_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetProjectError {
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::get_projects`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum GetProjectsError {
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::modify_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ModifyGalleryImageError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::modify_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ModifyProjectError {
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("The requested item(s) were not found or no authorization to access the requested item(s)")]
    Status404,
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::patch_projects`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum PatchProjectsError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::random_projects`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum RandomProjectsError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::schedule_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum ScheduleProjectError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::search_projects`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum SearchProjectsError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}
/// struct for typed errors of method [`ProjectsApi::unfollow_project`]
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(untagged)]
pub enum UnfollowProjectError {
    #[error("Request was invalid, see given error")]
    Status400(models::InvalidInputError),
    #[error("Incorrect token scopes or no authorization to access the requested item(s)")]
    Status401(models::AuthError),
    #[error("Unrecognized Error")]
    Unknown(serde_json::Value),
}

pub struct ProjectsApi<'c>(pub(crate) &'c crate::ApiClient);
impl<'c> ProjectsApi<'c> {
    /// Modrinth allows you to upload files of up to 5MiB to a project's gallery.
    pub async fn add_gallery_image(&self, params: &AddGalleryImageParams<'_,'_,'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let AddGalleryImageParams { mod_id, ext, featured, title, description, ordering, body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format_args!(
            "/project/{id_slug}/gallery"
            , id_slug=crate::urlencode(mod_id)
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

        local_var_req_builder = local_var_req_builder.query(&[("ext", ext)]);

        local_var_req_builder = local_var_req_builder.query(&[("featured", featured)]);

        if let Some(ref title) = title {
            local_var_req_builder = local_var_req_builder.query(&[("title", title)]);
        }

        if let Some(ref description) = description {
            local_var_req_builder = local_var_req_builder.query(&[("description", description)]);
        }

        if let Some(ref ordering) = ordering {
            local_var_req_builder = local_var_req_builder.query(&[("ordering", ordering)]);
        }
        local_var_req_builder = local_var_req_builder.json(body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => AddGalleryImageError::Status400(serde_json::from_str(&local_var_content)?),
                401 => AddGalleryImageError::Status401(serde_json::from_str(&local_var_content)?),
                404 => AddGalleryImageError::Status404,
                _ => AddGalleryImageError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    /// The new icon may be up to 256KiB in size.
    pub async fn change_project_icon(&self, params: &ChangeProjectIconParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ChangeProjectIconParams { mod_id, ext, body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            format_args!(
            "/project/{id_slug}/icon"
            , id_slug=crate::urlencode(mod_id)
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

        local_var_req_builder = local_var_req_builder.query(&[("ext", ext)]);
        local_var_req_builder = local_var_req_builder.json(body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => ChangeProjectIconError::Status400(serde_json::from_str(&local_var_content)?),
                _ => ChangeProjectIconError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn check_project_validity(&self, params: &CheckProjectValidityParams<'_,>) -> Result<models::ProjectIdentifier> {
        // unwrap the parameters
        let CheckProjectValidityParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/project/{id_slug}/check"
            , id_slug=crate::urlencode(mod_id)
            )
        );

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => CheckProjectValidityError::Status404,
                _ => CheckProjectValidityError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn create_project(&self, params: &CreateProjectParams<'_,>) -> Result<models::Project> {
        // unwrap the parameters
        let CreateProjectParams { data, icon, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            "/project"
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
        let mut local_var_form = reqwest::multipart::Form::new();
        local_var_form = local_var_form.text("data", serde_json::to_string(data)?);
        if let Some(ref _icon) = icon {
            // FIXME
            return Err(crate::ErrorKind::Other("File uploads not yet supported".into()))?;
        }
        local_var_req_builder = local_var_req_builder.multipart(local_var_form);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => CreateProjectError::Status400(serde_json::from_str(&local_var_content)?),
                401 => CreateProjectError::Status401(serde_json::from_str(&local_var_content)?),
                _ => CreateProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn delete_gallery_image(&self, params: &DeleteGalleryImageParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let DeleteGalleryImageParams { mod_id, url, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/project/{id_slug}/gallery"
            , id_slug=crate::urlencode(mod_id)
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

        local_var_req_builder = local_var_req_builder.query(&[("url", url)]);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => DeleteGalleryImageError::Status400(serde_json::from_str(&local_var_content)?),
                401 => DeleteGalleryImageError::Status401(serde_json::from_str(&local_var_content)?),
                _ => DeleteGalleryImageError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn delete_project(&self, params: &DeleteProjectParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let DeleteProjectParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/project/{id_slug}"
            , id_slug=crate::urlencode(mod_id)
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
                400 => DeleteProjectError::Status400(serde_json::from_str(&local_var_content)?),
                401 => DeleteProjectError::Status401(serde_json::from_str(&local_var_content)?),
                _ => DeleteProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn delete_project_icon(&self, params: &DeleteProjectIconParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let DeleteProjectIconParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/project/{id_slug}/icon"
            , id_slug=crate::urlencode(mod_id)
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
                400 => DeleteProjectIconError::Status400(serde_json::from_str(&local_var_content)?),
                401 => DeleteProjectIconError::Status401(serde_json::from_str(&local_var_content)?),
                _ => DeleteProjectIconError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn follow_project(&self, params: &FollowProjectParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let FollowProjectParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format_args!(
            "/project/{id_slug}/follow"
            , id_slug=crate::urlencode(mod_id)
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
                400 => FollowProjectError::Status400(serde_json::from_str(&local_var_content)?),
                401 => FollowProjectError::Status401(serde_json::from_str(&local_var_content)?),
                _ => FollowProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn get_dependencies(&self, params: &GetDependenciesParams<'_,>) -> Result<models::ProjectDependencyList> {
        // unwrap the parameters
        let GetDependenciesParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/project/{id_slug}/dependencies"
            , id_slug=crate::urlencode(mod_id)
            )
        );

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetDependenciesError::Status404,
                _ => GetDependenciesError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_project(&self, params: &GetProjectParams<'_,>) -> Result<models::Project> {
        // unwrap the parameters
        let GetProjectParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            format_args!(
            "/project/{id_slug}"
            , id_slug=crate::urlencode(mod_id)
            )
        );

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                404 => GetProjectError::Status404,
                _ => GetProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn get_projects(&self, params: &GetProjectsParams<'_,>) -> Result<Vec<models::Project>> {
        // unwrap the parameters
        let GetProjectsParams { ids, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/projects"
        );

        local_var_req_builder = local_var_req_builder.query(&[("ids", serde_json::to_string(ids)?)]); // MANUAL CHANGE

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                _ => GetProjectsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn modify_gallery_image(&self, params: &ModifyGalleryImageParams<'_,'_,'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ModifyGalleryImageParams { mod_id, url, featured, title, description, ordering, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            format_args!(
            "/project/{id_slug}/gallery"
            , id_slug=crate::urlencode(mod_id)
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

        local_var_req_builder = local_var_req_builder.query(&[("url", url)]);

        if let Some(ref featured) = featured {
            local_var_req_builder = local_var_req_builder.query(&[("featured", featured)]);
        }

        if let Some(ref title) = title {
            local_var_req_builder = local_var_req_builder.query(&[("title", title)]);
        }

        if let Some(ref description) = description {
            local_var_req_builder = local_var_req_builder.query(&[("description", description)]);
        }

        if let Some(ref ordering) = ordering {
            local_var_req_builder = local_var_req_builder.query(&[("ordering", ordering)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                401 => ModifyGalleryImageError::Status401(serde_json::from_str(&local_var_content)?),
                404 => ModifyGalleryImageError::Status404,
                _ => ModifyGalleryImageError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn modify_project(&self, params: &ModifyProjectParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ModifyProjectParams { mod_id, editable_project, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            format_args!(
            "/project/{id_slug}"
            , id_slug=crate::urlencode(mod_id)
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
        local_var_req_builder = local_var_req_builder.json(editable_project);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                401 => ModifyProjectError::Status401(serde_json::from_str(&local_var_content)?),
                404 => ModifyProjectError::Status404,
                _ => ModifyProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn patch_projects(&self, params: &PatchProjectsParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let PatchProjectsParams { ids, patch_projects_body, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::PATCH,
            "/projects"
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

        local_var_req_builder = local_var_req_builder.query(&[("ids", serde_json::to_string(ids)?)]); // MANUAL CHANGE
        local_var_req_builder = local_var_req_builder.json(patch_projects_body);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => PatchProjectsError::Status400(serde_json::from_str(&local_var_content)?),
                401 => PatchProjectsError::Status401(serde_json::from_str(&local_var_content)?),
                _ => PatchProjectsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn random_projects(&self, params: &RandomProjectsParams<>) -> Result<Vec<models::Project>> {
        // unwrap the parameters
        let RandomProjectsParams { count, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/projects_random"
        );

        local_var_req_builder = local_var_req_builder.query(&[("count", count)]);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => RandomProjectsError::Status400(serde_json::from_str(&local_var_content)?),
                _ => RandomProjectsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn schedule_project(&self, params: &ScheduleProjectParams<'_,'_,>) -> Result<()> {
        // unwrap the parameters
        let ScheduleProjectParams { mod_id, schedule, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::POST,
            format_args!(
            "/project/{id_slug}/schedule"
            , id_slug=crate::urlencode(mod_id)
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
        local_var_req_builder = local_var_req_builder.json(schedule);

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => ScheduleProjectError::Status400(serde_json::from_str(&local_var_content)?),
                401 => ScheduleProjectError::Status401(serde_json::from_str(&local_var_content)?),
                _ => ScheduleProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

    pub async fn search_projects(&self, params: &SearchProjectsParams<'_,'_,'_,>) -> Result<models::SearchResults> {
        // unwrap the parameters
        let SearchProjectsParams { query, facets, index, offset, limit, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::GET,
            "/search"
        );

        if let Some(ref query) = query {
            local_var_req_builder = local_var_req_builder.query(&[("query", query)]);
        }

        if let Some(ref facets) = facets {
            local_var_req_builder = local_var_req_builder.query(&[("facets", facets)]);
        }

        if let Some(ref index) = index {
            local_var_req_builder = local_var_req_builder.query(&[("index", index)]);
        }

        if let Some(ref offset) = offset {
            local_var_req_builder = local_var_req_builder.query(&[("offset", offset)]);
        }

        if let Some(ref limit) = limit {
            local_var_req_builder = local_var_req_builder.query(&[("limit", limit)]);
        }

        let local_var_resp = local_var_req_builder.send().await?;

        let local_var_status = local_var_resp.status();
        let local_var_content = local_var_resp.text().await?;

        if local_var_status.is_client_error() || local_var_status.is_server_error() {
            #[allow(clippy::match_single_binding)]
            let local_var_error = match local_var_status.as_u16() {
                400 => SearchProjectsError::Status400(serde_json::from_str(&local_var_content)?),
                _ => SearchProjectsError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            serde_json::from_str(&local_var_content).map_err(Into::into)
        }
    }

    pub async fn unfollow_project(&self, params: &UnfollowProjectParams<'_,>) -> Result<()> {
        // unwrap the parameters
        let UnfollowProjectParams { mod_id, } = params;

        #[allow(unused_mut)]
        let mut local_var_req_builder = self.0.request(
            reqwest::Method::DELETE,
            format_args!(
            "/project/{id_slug}/follow"
            , id_slug=crate::urlencode(mod_id)
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
                400 => UnfollowProjectError::Status400(serde_json::from_str(&local_var_content)?),
                401 => UnfollowProjectError::Status401(serde_json::from_str(&local_var_content)?),
                _ => UnfollowProjectError::Unknown(serde_json::from_str(&local_var_content)?),
            };
            Err(ErrorResponse { status: local_var_status, content: local_var_content, source: Some(local_var_error.into()) }.into())
        } else {
            Ok(())
        }
    }

}
