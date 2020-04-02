use std::net::{Ipv4Addr, Ipv6Addr};

use cloudflare::endpoints::dns::{CreateDnsRecord, CreateDnsRecordParams, DeleteDnsRecord, DeleteDnsRecordResponse, DnsContent, DnsRecord, DnsRecordDetails, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams};
use cloudflare::framework::{
    apiclient::ApiClient,
    HttpApiClient,
};
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

pub struct CreateParams<'a> {
    pub zone_id: &'a str,
    pub name: &'a str,
    pub ttl: u32,
    pub proxied: bool,
    pub content: &'a str,
    pub record_type: &'a str,
    pub priority: u16,
}

pub struct UpdateParams<'a> {
    pub id: &'a str,
    pub zone_id: &'a str,
    pub name: Option<&'a str>,
    pub ttl: Option<u32>,
    pub proxied: Option<bool>,
    pub content: Option<&'a str>,
}

fn resolve_content(record_type: &str, content: &str, priority: u16) -> Result<DnsContent, &'static str> {
    match record_type {
        "A" => {
            let ip: Option<Ipv4Addr> = content.parse().ok();

            match ip {
                Some(content) => Ok(DnsContent::A { content }),
                None => Err("Invalid IPv4 address")
            }
        }
        "AAAA" => {
            let ip: Option<Ipv6Addr> = content.parse().ok();

            match ip {
                Some(content) => Ok(DnsContent::AAAA { content }),
                None => Err("Invalid IPv6 address")
            }
        }
        "CNAME" => Ok(DnsContent::CNAME { content: String::from(content) }),
        "MX" => Ok(DnsContent::MX { content: String::from(content), priority }),
        "TXT" => Ok(DnsContent::TXT { content: String::from(content) }),
        "NS" => Ok(DnsContent::CNAME { content: String::from(content) }),
        _ => Err("Record type not supported")
    }
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

pub fn create(api: &HttpApiClient, record: CreateParams) {
    let content = match resolve_content(record.record_type, record.content, record.priority) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let response = api.request(&CreateDnsRecord {
        zone_identifier: record.zone_id,
        params: CreateDnsRecordParams {
            ttl: Some(record.ttl),
            priority: None,
            proxied: Some(record.proxied),
            name: &record.name,
            content,
        },
    });

    match response {
        Ok(success) => {
            let record: DnsRecord = success.result;
            // @todo print complete record formatted
            println!("Record \"{}\" created", record.id)
        }
        // @todo abstract error formatting
        Err(failure) => println!("An error occurred {:?}", failure)
    }
}

pub fn delete(api: &HttpApiClient, zone_id: &str, ids: Vec<&str>) {
    for id in ids {
        let response = api.request(&DeleteDnsRecord {
            zone_identifier: zone_id,
            identifier: id,
        });

        match response {
            Ok(success) => {
                let record: DeleteDnsRecordResponse = success.result;
                println!("Record \"{}\" deleted", record.id)
            }
            // @todo abstract error formatting
            Err(failure) => println!("An error occurred {:?}", failure)
        }
    }
}

pub fn update(api: &HttpApiClient, input: UpdateParams) {
    let get_response = api.request(&DnsRecordDetails {
        zone_identifier: input.zone_id,
        identifier: input.id,
    });

    match get_response {
        Err(failure) => println!("{:?}", failure),
        Ok(success) => {
            let record: DnsRecord = success.result;

            let record_type = match record.content {
                DnsContent::A { content: _ } => "A",
                DnsContent::AAAA { content: _ } => "AAAA",
                DnsContent::CNAME { content: _ } => "CNAME",
                DnsContent::NS { content: _ } => "NS",
                DnsContent::MX { content: _, priority: _ } => "MX",
                DnsContent::TXT { content: _ } => "TXT",
            };

            let content = match input.content {
                Some(content) => {
                    match resolve_content(record_type, content, 1) {
                        Ok(resolved) => resolved,
                        Err(e) => {
                            println!("{}", e);
                            return;
                        }
                    }
                }
                None => record.content
            };

            let name: &str = match input.name {
                Some(n) => n,
                None => &record.name
            };
            let response = api.request(&UpdateDnsRecord {
                zone_identifier: input.zone_id,
                identifier: input.id,
                params: UpdateDnsRecordParams {
                    ttl: input.ttl.or(Some(record.ttl)),
                    proxied: input.proxied.or(Some(record.proxied)),
                    name,
                    content,
                },
            });

            match response {
                Ok(success) => {
                    let record: DnsRecord = success.result;
                    println!("Record {} updated", record.id)
                }
                // @todo abstract error formatting
                Err(failure) => println!("An error occurred {:?}", failure)
            }
        }
    }
}
