/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.240719
 * 
 * Generated by: https://openapi-generator.tech
 */

#[allow(unused_imports)]
use crate::models;

/// Relevant links for the mod such as Issue tracker and Wiki
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModLinks {
    #[serde(rename = "websiteUrl")]
    pub website_url: ::url::Url,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: ::url::Url,
    #[serde(rename = "issuesUrl")]
    pub issues_url: ::url::Url,
    #[serde(rename = "sourceUrl")]
    pub source_url: ::url::Url,
}

impl ModLinks {
    /// Relevant links for the mod such as Issue tracker and Wiki
    #[allow(clippy::too_many_arguments)]
    pub fn new(website_url: ::url::Url, wiki_url: ::url::Url, issues_url: ::url::Url, source_url: ::url::Url) -> Self {
        Self {
            website_url,
            wiki_url,
            issues_url,
            source_url,
        }
    }
}

