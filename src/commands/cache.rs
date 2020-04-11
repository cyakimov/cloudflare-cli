use cloudflare::framework::{
    apiclient::ApiClient,
    HttpApiClient,
};
use crate::api::endpoints::cache::{PurgeCache, PurgeCacheParams};
use crate::{terminal, http};
use clap::Values;
use cloudflare::framework::response::ApiResponse;

pub fn process_response<T>(res: ApiResponse<T>) {
    match res {
        Ok(_) => terminal::info("Successfully purged assets. Please allow up to 30 seconds for changes to take effect."),
        Err(e) => terminal::error(http::format_error(e, None).as_str())
    }
}

pub fn purge_all(api: &HttpApiClient, zone_id: &str) {
    let res = api.request(&PurgeCache {
        zone_identifier: zone_id,
        params: PurgeCacheParams {
            purge_everything: Some(true),
            files: None,
        },
    });

    process_response(res);
}

pub fn purge_url(api: &HttpApiClient, zone_id: &str, urls: Values) {
    let files: Vec<String> = urls.map(String::from).collect();

    let res = api.request(&PurgeCache {
        zone_identifier: zone_id,
        params: PurgeCacheParams {
            purge_everything: None,
            files: Some(files),
        },
    });

    process_response(res);
}
