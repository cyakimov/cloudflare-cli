use cloudflare::framework::{
    apiclient::ApiClient,
    HttpApiClient,
};
use cloudflare::endpoints::dns::{DnsRecord, DnsContent, CreateDnsRecord, ListDnsRecords, ListDnsRecordsParams, CreateDnsRecordParams};
use tabular::{Row};
use crate::commands::table_from_cols;

pub fn list(api: &HttpApiClient, zone_id: &str, page: u32, limit: u32) {
    let response = api.request(&ListDnsRecords {
        zone_identifier: zone_id,
        params: ListDnsRecordsParams {
            record_type: None,
            name: None,
            page: Some(page),
            per_page: Some(limit),
            order: None,
            direction: None,
            search_match: None,
        },
    });

    match response {
        Ok(success) => {
            let list: Vec<DnsRecord> = success.result;
            let columns = vec![
                "ID",
                "NAME",
                "TYPE",
                "CONTENT",
                "TTL",
                "PROXY",
            ];
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

                row.add_cell(record.ttl).add_cell(if record.proxied { "Yes" } else { "No" });

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
