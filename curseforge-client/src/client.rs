use crate::apis;

use reqwest::Client;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AuthData {
    pub api_key_auth: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    server: reqwest::Url,
    client: Client,
    pub(crate) auth: AuthData,
}

impl ApiClient {
    pub fn builder() -> Builder { Builder::default() }

    /// `path` should always start with single `/` and will be appended to the base server url
    pub fn request(&self, method: reqwest::Method, path: impl ::std::fmt::Display) -> reqwest::RequestBuilder {
        let base = self.server.as_str().strip_suffix('/').unwrap_or(self.server.as_str());
        self.client.request(method, format!("{base}{path}"))
    }

    pub fn categories(&self) -> apis::CategoriesApi {
        apis::CategoriesApi(&self)
    }

    pub fn files(&self) -> apis::FilesApi {
        apis::FilesApi(&self)
    }

    pub fn fingerprints(&self) -> apis::FingerprintsApi {
        apis::FingerprintsApi(&self)
    }

    pub fn games(&self) -> apis::GamesApi {
        apis::GamesApi(&self)
    }

    pub fn minecraft(&self) -> apis::MinecraftApi {
        apis::MinecraftApi(&self)
    }

    pub fn mods(&self) -> apis::ModsApi {
        apis::ModsApi(&self)
    }

}

impl Default for ApiClient {
    fn default() -> Self {
        Self::builder().build().expect("should successfully initialize with defaults")
    }
}

#[derive(Default)]
pub struct Builder {
    server: Option<reqwest::Url>,
    user_agent: Option<String>,
    auth: Option<AuthData>,
}

const DEFAULT_SERVER: std::cell::OnceCell<reqwest::Url> = std::cell::OnceCell::new();
const DEFAULT_AGENT: &str = concat!("OpenAPI-Generator/", env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
impl Builder {
    pub fn build(self) -> crate::Result<ApiClient> {
        let client = Client::builder().user_agent(self.user_agent.as_deref().unwrap_or(DEFAULT_AGENT)).build()?;
        Ok(ApiClient {
            server: self.server.unwrap_or_else(|| {
                DEFAULT_SERVER.get_or_init(|| "https://api.curseforge.com".parse().expect("should have valid default server")).to_owned()
            }),
            client: client,
            auth: self.auth.unwrap_or_default(),
        })
    }

    pub fn server(mut self, server: impl Into<reqwest::Url>) -> Self {
        self.server = Some(server.into());
        self
    }

    pub fn user_agent(mut self, user_agent: impl ToString) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn auth(mut self, auth: AuthData) -> Self {
        self.auth = Some(auth);
        self
    }
}
