/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FingerprintMatchesResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<models::FingerprintMatch>,
    #[serde(rename = "exactFingerprints")]
    pub exact_fingerprints: Vec<u64>,
    #[serde(rename = "partialMatches")]
    pub partial_matches: Vec<models::FingerprintMatch>,
    #[serde(rename = "partialMatchFingerprints")]
    pub partial_match_fingerprints: serde_json::Value,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Vec<u64>,
    #[serde(rename = "installedFingerprints")]
    pub installed_fingerprints: Vec<u64>,
    #[serde(rename = "unmatchedFingerprints")]
    pub unmatched_fingerprints: Vec<u64>,
}

impl FingerprintMatchesResult {
    pub fn new(is_cache_built: bool, exact_matches: Vec<models::FingerprintMatch>, exact_fingerprints: Vec<u64>, partial_matches: Vec<models::FingerprintMatch>, partial_match_fingerprints: serde_json::Value, additional_properties: Vec<u64>, installed_fingerprints: Vec<u64>, unmatched_fingerprints: Vec<u64>) -> Self {
        Self {
            is_cache_built,
            exact_matches,
            exact_fingerprints,
            partial_matches,
            partial_match_fingerprints,
            additional_properties,
            installed_fingerprints,
            unmatched_fingerprints,
        }
    }
}

