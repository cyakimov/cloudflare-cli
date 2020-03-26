use cloudflare::framework::{
    apiclient::ApiClient,
    HttpApiClient,
};
use cloudflare::endpoints::dns::{DnsRecord, DnsContent, CreateDnsRecord, ListDnsRecords, ListDnsRecordsParams, CreateDnsRecordParams};
use tabular::Row;
use crate::commands::table_from_cols;

pub struct ListParams<'a, 'b> {
    pub zone_id: &'a str,
    pub page: u32,
    pub limit: u32,
    pub wide: bool,
    pub filters: ListFilters<'b>,
}

pub struct ListFilters<'a> {
    pub all: Option<&'a str>
}

pub fn list(api: &HttpApiClient, params: ListParams) {
    let name = match params.filters.all {
        Some(n) => Some(format!("contains:{}", n)),
        _ => None
    };

    let response = api.request(&ListDnsRecords {
        zone_identifier: params.zone_id,
        params: ListDnsRecordsParams {
            record_type: None,
            name,
            page: Some(params.page),
            per_page: Some(params.limit),
            order: None,
            direction: None,
            search_match: None,
        },
    });

    match response {
        Ok(success) => {
            let list: Vec<DnsRecord> = success.result;

            let columns = if params.wide {
                vec![
                    "ID",
                    "NAME",
                    "TYPE",
                    "CONTENT",
                    "TTL",
                    "PROXY",
                    "LOCKED",
                    "CREATED",
                    "MODIFIED",
                ]
            } else {
                vec![
                    "ID",
                    "NAME",
                    "TYPE",
                    "CONTENT",
                    "TTL",
                    "PROXY",
                ]
            };

            let mut table = table_from_cols(columns);

            for record in list {
                let mut row = Row::new().with_cell(record.id).with_cell(record.name);

                match record.content {
                    DnsContent::A { content: c } => row.add_cell("A").add_cell(c),
                    DnsContent::AAAA { content: c } => row.add_cell("AAAA").add_cell(c),
                    DnsContent::CNAME { content: c } => row.add_cell("CNAME").add_cell(c),
                    DnsContent::NS { content: c } => row.add_cell("NS").add_cell(c),
                    DnsContent::MX { content: c, priority: _ } => row.add_cell("MX").add_cell(c),
                    DnsContent::TXT { content: c } => row.add_cell("TXT").add_cell(c),
                };

                let ttl = format!("{}", record.ttl);
                row.add_cell(if ttl == "1" { "Auto" } else { &ttl })
                    .add_cell(if record.proxied { "Yes" } else { "No" });

                if params.wide {
                    row.add_cell(if record.locked { "Yes" } else { "No" })
                        .add_cell(record.created_on)
                        .add_cell(record.modified_on);
                }

                table.add_row(row);
            }
            print!("{}", table);
        }
        Err(e) => println!("{:?}", e)
    }
}

pub fn create(api: &HttpApiClient, zone_id: &str, record: DnsRecord) {
    let response = api.request(&CreateDnsRecord {
        zone_identifier: zone_id,
        params: CreateDnsRecordParams {
            ttl: Some(record.ttl),
            priority: None,
            proxied: Some(record.proxied),
            name: &record.name,
            content: record.content,
        },
    });

    match response {
        Ok(success) => {
            let record: DnsRecord = success.result;
            println!("DNS Record has been created with ID '{}'", record.id)
        }
        Err(failure) => println!("An error occurred {:?}", failure)
    }
}
