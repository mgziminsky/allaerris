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
use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FingerprintMatchesResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches", default)]
    pub exact_matches: Vec<models::FingerprintMatch>,
    #[serde(rename = "exactFingerprints", default)]
    pub exact_fingerprints: Vec<u32>,
    #[serde(rename = "partialMatches", default)]
    pub partial_matches: Vec<models::FingerprintMatch>,
    #[serde(rename = "partialMatchFingerprints")]
    pub partial_match_fingerprints: std::collections::HashMap<String, Vec<i32>>,
    #[serde(rename = "installedFingerprints", default)]
    pub installed_fingerprints: Vec<u32>,
    #[serde(rename = "unmatchedFingerprints", default)]
    pub unmatched_fingerprints: Vec<u32>,
}

impl FingerprintMatchesResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(is_cache_built: bool, exact_matches: Vec<models::FingerprintMatch>, exact_fingerprints: Vec<u32>, partial_matches: Vec<models::FingerprintMatch>, partial_match_fingerprints: std::collections::HashMap<String, Vec<i32>>, installed_fingerprints: Vec<u32>, unmatched_fingerprints: Vec<u32>) -> Self {
        Self {
            is_cache_built,
            exact_matches,
            exact_fingerprints,
            partial_matches,
            partial_match_fingerprints,
            installed_fingerprints,
            unmatched_fingerprints,
        }
    }
}

