use cloudflare::framework::endpoint::{Endpoint, Method};
use cloudflare::framework::response::ApiResult;

/// Remove files from Cloudflare cache
/// https://api.cloudflare.com/#zone-purge-all-files
pub struct PurgeCache<'a> {
    pub zone_identifier: &'a str,
    pub params: PurgeCacheParams,
}

impl<'a> Endpoint<Cache, (), PurgeCacheParams> for PurgeCache<'a> {
    fn method(&self) -> Method {
        Method::Post
    }
    fn path(&self) -> String {
        format!("zones/{}/purge_cache", self.zone_identifier)
    }
    fn body(&self) -> Option<PurgeCacheParams> {
        Some(self.params.clone())
    }
}

#[derive(Debug, Deserialize)]
pub struct Cache {
    pub id: String
}

#[derive(Clone, Serialize)]
pub struct PurgeCacheParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purge_everything: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
}

impl ApiResult for Cache {}
