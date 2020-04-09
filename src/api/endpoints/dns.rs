use cloudflare::framework::endpoint::{Endpoint, Method};
use cloudflare::endpoints::dns::DnsRecord;

/// DNS Record Details
/// https://api.cloudflare.com/#dns-records-for-a-zone-dns-record-details
pub struct DnsRecordDetails<'a> {
    pub zone_identifier: &'a str,
    pub identifier: &'a str,
}
impl<'a> Endpoint<DnsRecord> for DnsRecordDetails<'a> {
    fn method(&self) -> Method {
        Method::Get
    }
    fn path(&self) -> String {
        format!("zones/{}/dns_records/{}", self.zone_identifier, self.identifier)
    }
}
