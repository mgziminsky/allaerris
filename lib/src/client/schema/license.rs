use url::Url;

#[derive(Debug, Clone)]
pub struct License {
    pub name: String,
    pub spdx_id: String,
    pub url: Option<Url>,
}
