use url::Url;

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub url: Option<Url>
}
