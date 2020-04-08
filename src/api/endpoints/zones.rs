use chrono::DateTime;
use chrono::offset::Utc;
use cloudflare::endpoints::{
    zone::{
        HostingPartner,
        ListZonesParams,
        Owner,
        Status,
        Type,
    },
};
use cloudflare::framework::{
    endpoint::{Endpoint, Method},
    response::ApiResult,
};
use crate::api::endpoints::plan::Plan;
use crate::api::endpoints::account::Account;

// Workaround for error E0117
#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ZoneVec {
    pub zones: Vec<Zone>
}

/// List Zones
/// List, search, sort, and filter your zones
/// https://api.cloudflare.com/#zone-list-zones
pub struct ListZones {
    pub params: ListZonesParams
}

impl Endpoint<ZoneVec, ListZonesParams> for ListZones {
    fn method(&self) -> Method {
        Method::Get
    }
    fn path(&self) -> String {
        "zones".to_string()
    }
    fn query(&self) -> Option<ListZonesParams> {
        Some(self.params.clone())
    }
}

/// Extra Cloudflare-specific information about the record
#[derive(Deserialize, Debug)]
pub struct Meta {
    /// Will exist if Cloudflare automatically added this DNS record during initial setup.
    pub auto_added: Option<bool>,
}

/// A Zone is a domain name along with its subdomains and other identities
/// https://api.cloudflare.com/#zone-properties
#[derive(Deserialize, Debug)]
pub struct Zone {
    /// Zone identifier tag
    pub id: String,
    /// The domain name
    pub name: String,
    /// Information about the account the zone belongs to
    pub account: Account,
    /// A list of beta features in which the zone is participating
    pub betas: Option<Vec<String>>,
    /// When the zone was created
    pub created_on: DateTime<Utc>,
    /// Exists only with a deactivated status and indicates the reason the zone is not resolving on
    /// the Cloudflare network.
    pub deactivation_reason: Option<String>,
    /// The interval (in seconds) from when development mode expires (positive integer) or last
    /// expired (negative integer) for the domain. If development mode has never been enabled, this
    /// value is 0.
    pub development_mode: u8,
    /// Hosting partner information, if the zone signed up via a Cloudflare hosting partner
    pub host: Option<HostingPartner>,
    /// Metadata about the domain.
    pub meta: Meta,
    /// When the zone was last modified
    pub modified_on: DateTime<Utc>,
    /// Cloudflare-assigned name servers. This is only populated for zones that use Cloudflare DNS
    pub name_servers: Vec<String>,
    /// DNS host at the time of switching to Cloudflare
    pub original_dnshost: Option<String>,
    /// Original name servers before moving to Cloudflare
    pub original_name_servers: Option<Vec<String>>,
    /// Registrar for the domain at the time of switching to Cloudflare
    pub original_registrar: Option<String>,
    /// Information about the owner of the zone
    pub owner: Owner,
    /// Indicates if the zone is only using Cloudflare DNS services. A true value means the zone
    /// will not receive security or performance benefits.
    pub paused: bool,
    /// Available permissions on the zone for the current user requesting the item
    pub permissions: Vec<String>,
    /// A zone plan
    pub plan: Option<Plan>,
    /// A zone plan
    pub plan_pending: Option<Plan>,
    /// Status of the zone
    pub status: Status,
    /// An array of domains used for custom name servers. This is only available for Business and
    /// Enterprise plans.
    pub vanity_name_servers: Option<Vec<String>>,
    /// A full zone implies that DNS is hosted with Cloudflare. A partial zone is typically a
    /// partner-hosted zone or a CNAME setup.
    #[serde(rename = "type")]
    pub zone_type: Type,
}

impl ApiResult for Zone {}
impl ApiResult for ZoneVec {}
