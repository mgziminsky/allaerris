/*
 * Labrinth
 *
 * This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:  - [cURL](https://curl.se/) (recommended, command-line) - [ReqBIN](https://reqbin.com/) (recommended, online) - [Postman](https://www.postman.com/downloads/) - [Insomnia](https://insomnia.rest/) - Your web browser, if you don't need to send headers or a request body  Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:  ```json {   \"about\": \"Welcome traveler!\",   \"documentation\": \"https://docs.modrinth.com\",   \"name\": \"modrinth-labrinth\",   \"version\": \"2.7.0\" } ```  If you got a response similar to the one above, you can use the Modrinth API! When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.  ## Authentication This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth). All tokens are tied to a Modrinth user and use the `Authorization` header of the request.  Example: ``` Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic ```  You do not need a token for most requests. Generally speaking, only the following types of requests require a token: - those which create data (such as version creation) - those which modify data (such as editing a project) - those which access private data (such as draft projects, notifications, emails, and payout data)  Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope. You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/v3/pats.rs#L17). Making a request with an invalid scope will return a 401 error.  Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth. For example, deleting a user account can only be done through Modrinth's frontend.  ### OAuth2 Applications interacting with the authenticated API should create an OAuth2 application. You can do this in [the developer settings](https://modrinth.com/settings/applications).  Once you have created a client, use the following URL to have a user authorize your client: ``` https://modrinth.com/auth/authorize?client_id=<CLIENT_ID>&redirect_uri=<CALLBACK_URL>&scope=<SCOPE_ONE>+<SCOPE_TWO>+<SCOPE_THREE> ```  Then, use the following URL to get the token: ``` https://api.modrinth.com/_internal/oauth/token ```  This route will be changed in the future to move the `_internal` part to `v3`.  ### Personal access tokens Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).  ### GitHub tokens For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes. **We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.** GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.  ## Cross-Origin Resource Sharing This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/). This allows for cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.  ## Identifiers The majority of items you can interact with in the API have a unique eight-digit base62 ID. Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves. Version files use the sha1 or sha512 file hashes as identifiers.  Each project and user has a friendlier way of identifying them; slugs and usernames, respectively. While unique IDs are constant, slugs and usernames can change at any moment. If you want to store something in the long term, it is recommended to use the unique ID.  ## Ratelimits The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers. - `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute - `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window - `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets  Ratelimits are the same no matter whether you use a token or not. The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).  ## User Agents To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header. Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic. It is recommended, but not required, to include contact information in your user agent. This allows us to contact you if we would like a change in your application's behavior without having to block your traffic. - Bad: `User-Agent: okhttp/4.9.3` - Good: `User-Agent: project_name` - Better: `User-Agent: github_username/project_name/1.56.0` - Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`  ## Versioning Modrinth follows a simple pattern for its API versioning. In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.  When an API is no longer the current one, it will immediately be considered deprecated. No more support will be provided for API versions older than the current one. It will be kept for some time, but this amount of time is not certain.  We will exercise various tactics to get people to update their implementation of our API. One example is by adding something like `STOP USING THIS API` to various data returned by the API.  Once an API version is completely deprecated, it will permanently return a 410 error. Please ensure your application handles these 410 errors.  ### Migrations Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.  Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.  <details><summary>API v1 to API v2</summary>  These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.  - The search route has been moved from `/api/v1/mod` to `/v2/search` - New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery` - New search facet: `project_type` - Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch) - New search fields: `project_type`, `gallery`   - The gallery field is an array of URLs to images that are part of the project's gallery - The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page - Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.   - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth - In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft   - Similarly, for version creation, a version may not be uploaded without any files - Donation URLs have been enabled - New project status: `archived`. Projects with this status do not appear in search - Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached - Dependencies have been wiped and replaced with a new system - Notifications now have a `type` field, such as `project_update`  Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.  </details><details><summary>Minotaur v1 to Minotaur v2</summary>  Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.  First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.  To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.  Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.  Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.  You may now use the slug anywhere that a project ID was previously required.  </details> 
 *
 * The version of the OpenAPI document: v2.7.0/15cf3fc
 * Contact: support@modrinth.com
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NonSearchProject {
    /// The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$```
    #[serde(rename = "slug")]
    pub slug: String,
    /// The title or name of the project
    #[serde(rename = "title")]
    pub title: String,
    /// A short description of the project
    #[serde(rename = "description")]
    pub description: String,
    /// A list of the categories that the project has
    #[serde(rename = "categories", default)]
    pub categories: Vec<String>,
    /// The client side support of the project
    #[serde(rename = "client_side")]
    pub client_side: ClientSide,
    /// The server side support of the project
    #[serde(rename = "server_side")]
    pub server_side: ServerSide,
    /// A long form description of the project
    #[serde(rename = "body")]
    pub body: String,
    /// The status of the project
    #[serde(rename = "status")]
    pub status: Status,
    /// The requested status when submitting for review or scheduling the project for release
    #[serde(rename = "requested_status", skip_serializing_if = "Option::is_none")]
    pub requested_status: Option<RequestedStatus>,
    /// A list of categories which are searchable but non-primary
    #[serde(rename = "additional_categories", default)]
    pub additional_categories: Vec<String>,
    /// An optional link to where to submit bugs or issues with the project
    #[serde(rename = "issues_url", skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<String>,
    /// An optional link to the source code of the project
    #[serde(rename = "source_url", skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// An optional link to the project's wiki page or other relevant information
    #[serde(rename = "wiki_url", skip_serializing_if = "Option::is_none")]
    pub wiki_url: Option<String>,
    /// An optional invite link to the project's discord
    #[serde(rename = "discord_url", skip_serializing_if = "Option::is_none")]
    pub discord_url: Option<String>,
    /// A list of donation links for the project
    #[serde(rename = "donation_urls", default)]
    pub donation_urls: Vec<models::ProjectDonationUrl>,
}

impl NonSearchProject {
    pub fn new(slug: String, title: String, description: String, categories: Vec<String>, client_side: ClientSide, server_side: ServerSide, body: String, status: Status, additional_categories: Vec<String>, donation_urls: Vec<models::ProjectDonationUrl>) -> Self {
        Self {
            slug,
            title,
            description,
            categories,
            client_side,
            server_side,
            body,
            status,
            additional_categories,
            donation_urls,
            requested_status: None,
            issues_url: None,
            source_url: None,
            wiki_url: None,
            discord_url: None,
        }
    }
}

/// The client side support of the project
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum ClientSide {
    #[default]
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "unknown")]
    Unknown,
}

/// The server side support of the project
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum ServerSide {
    #[default]
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "unknown")]
    Unknown,
}

/// The status of the project
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Status {
    #[default]
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "unlisted")]
    Unlisted,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "withheld")]
    Withheld,
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "unknown")]
    Unknown,
}

/// The requested status when submitting for review or scheduling the project for release
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum RequestedStatus {
    #[default]
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "unlisted")]
    Unlisted,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "draft")]
    Draft,
}

