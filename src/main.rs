use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use cloudflare::endpoints::zone::ListZonesParams;
#[allow(unused_imports)]
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    Environment,
    HttpApiClient,
    HttpApiClientConfig,
};
use cloudflare::framework::response::ApiResponse;

use cflare::commands::{accounts, config, dns, zones, cache};
use cflare::config::{Config, get_global_config_path};
use cflare::api::endpoints::zones::{ListZones, ZoneVec};
use cflare::terminal;

const MAX_DNS_TTL: u32 = 2_147_483_647;

fn valid_u32(arg: String) -> Result<(), String> {
    match arg.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value must be an integer; received: {}", arg))
    }
}

fn valid_priority(arg: String) -> Result<(), String> {
    match arg.parse::<u16>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value must be an 16-bit integer; received: {}", arg))
    }
}

fn valid_ttl(arg: String) -> Result<(), String> {
    let ttl = arg.parse::<u32>();

    match ttl {
        Ok(value) => {
            if value < 1 || value > MAX_DNS_TTL {
                return Err(String::from(format!("Value must be between 1 and {} seconds", MAX_DNS_TTL)));
            }
            Ok(())
        }
        Err(_) => Err(format!("Value must be an integer; received: {}", arg))
    }
}

fn resolve_zone(api: &HttpApiClient, arg: &ArgMatches) -> String {
    if arg.is_present("zone-id") { arg.value_of("zone-id").unwrap().to_owned() } else {
        let zone = arg.value_of("zone").unwrap();
        let res: ApiResponse<ZoneVec> = api.request(&ListZones {
            params: ListZonesParams {
                name: Some(String::from(zone)),
                status: None,
                page: None,
                per_page: Some(1),
                order: None,
                direction: None,
                search_match: None,
            }
        });

        match res {
            Ok(success) => {
                let res: ZoneVec = success.result;
                let zones = res.zones;
                match zones.len() {
                    1 => zones[0].id.clone(),
                    _ => {
                        terminal::error(format!("Zone \"{}\" not found", zone).as_str());
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                println!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn get_api_client(args: &ArgMatches) -> HttpApiClient {
    let credentials: Credentials;
    let config_file = get_global_config_path().unwrap();
    let cred_flags = args.is_present("email") || args.is_present("key") || args.is_present("token");
    if !config_file.exists() && !cred_flags {
        terminal::warn("Config file does not exist. Try running `cflare config`");
        std::process::exit(1);
    }

    // Set credentials from flags/env
    if cred_flags {
        let email = args.value_of("email");
        let key = args.value_of("key");
        let token = args.value_of("token");

        credentials = if let Some(key) = key {
            Credentials::UserAuthKey {
                email: email.unwrap().to_string(),
                key: key.to_string(),
            }
        } else if let Some(token) = token {
            Credentials::UserAuthToken {
                token: token.to_string(),
            }
        } else {
            terminal::error("Either API token or API key + email pair must be provided");
            std::process::exit(1);
        };
    } else {
        let config: Config = match Config::from_file(config_file) {
            Ok(c) => c,
            Err(e) => {
                terminal::error(format!("{}", e).as_str());
                std::process::exit(1);
            }
        };
        let cred = &config.contexts[0].credential;
        credentials = Credentials::from(cred.to_owned());
    }

    HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    ).unwrap()
}

fn main() {
    let auth_args = vec![
        Arg::with_name("email")
            .global(true)
            .long("email")
            .short("e")
            .help("Email address associated with your account")
            .takes_value(true),
        Arg::with_name("key")
            .global(true)
            .long("key")
            .short("k")
            .help("API token generated on the \"My Account\" page")
            .takes_value(true),
        Arg::with_name("token")
            .global(true)
            .long("token")
            .short("t")
            .help("API token generated on the \"My Account\" page")
            .takes_value(true),
    ];
    let zone = Arg::with_name("zone")
        .help("Zone name. e.g. mydomain.com")
        .long("zone")
        .required_unless("zone-id")
        .takes_value(true);
    let zone_id = Arg::with_name("zone-id")
        .long("zone-id")
        .short("z")
        .takes_value(true)
        .conflicts_with("zone");

    let zone_args = [zone, zone_id];

    let limit = Arg::with_name("limit")
        .short("l")
        .long("limit")
        .validator(valid_u32)
        .takes_value(true);
    let record_type = Arg::with_name("type")
        .long("type")
        .takes_value(true)
        .possible_values(&["A", "AAAA", "CNAME", "MX", "TXT", "NS"]);

    let commands = vec![
        SubCommand::with_name("config").help("Setup your Cloudflare account"),
        SubCommand::with_name("accounts")
            .subcommands(vec![
                SubCommand::with_name("list").arg(
                    limit.clone()
                ),
                SubCommand::with_name("describe"),
            ]),
        SubCommand::with_name("zones")
            .subcommands(vec![
                SubCommand::with_name("list")
                    .arg(limit.clone()),
            ]),
        SubCommand::with_name("cache")
            .subcommands(vec![
                SubCommand::with_name("purge")
                    .args(&zone_args.clone())
                    .arg(Arg::with_name("all")
                        .short("A")
                        .long("all")
                        .help("Remove ALL files from Cloudflare's cache")
                    )
                    .arg(Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .multiple(true)
                        .max_values(30)
                        .required_unless("all")
                        .conflicts_with("all")
                        .help("Remove one or more files from Cloudflare's cache by specifying URLs")
                    )
            ]),
        SubCommand::with_name("dns")
            .subcommands(vec![
                SubCommand::with_name("list")
                    .args(&zone_args.clone())
                    .arg(Arg::with_name("wide").long("wide").short("w"))
                    .arg(Arg::with_name("name").long("name").short("n")
                        .takes_value(true).help("Filter by name. Performs partial matching"))
                    .arg(limit.clone()),
                SubCommand::with_name("delete")
                    .args(&zone_args.clone())
                    .arg(
                        Arg::with_name("id")
                            .required(true)
                            .min_values(1)
                            .help("Record identifier. Multiple values can be provided")
                    ),
                SubCommand::with_name("create")
                    .arg(Arg::with_name("name")
                        .takes_value(true)
                        .required(true)
                        .help("DNS record name")
                    )
                    .args(&zone_args.clone())
                    .arg(Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .takes_value(true)
                        .required(true)
                        .help("DNS record content")
                    )
                    .arg(record_type.clone().required(true))
                    .arg(Arg::with_name("ttl")
                        .long("ttl")
                        .validator(valid_ttl)
                        .takes_value(true)
                        .required(true)
                        .help("Time to live for DNS record. Value of 1 is 'automatic'")
                    )
                    .arg(Arg::with_name("proxied")
                        .long("proxied")
                        .help("Whether the record would be proxied by Cloudflare")
                    )
                    .arg(Arg::with_name("priority")
                        .long("priority")
                        .validator(valid_priority)
                        .takes_value(true)
                        .help("Used with some records like MX and SRV to determine priority")
                    ),
                SubCommand::with_name("update")
                    .arg(Arg::with_name("id")
                        .takes_value(true)
                        .help("DNS record id")
                        .required(true)
                    )
                    .arg(Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                        .help("DNS record name")
                    )
                    .args(&zone_args.clone())
                    .arg(Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .takes_value(true)
                        .help("DNS record content")
                    )
                    .arg(Arg::with_name("ttl")
                        .long("ttl")
                        .validator(valid_ttl)
                        .takes_value(true)
                        .help("Time to live for DNS record. Value of 1 is 'automatic'")
                    )
                    .arg(Arg::with_name("proxied")
                        .long("proxied")
                        .takes_value(true)
                        .possible_values(&["0", "1", "true", "false"])
                        .help("Whether the record would be proxied by Cloudflare")
                    )
            ]),
    ];

    let app = App::new("cflare")
        .name(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommands(commands)
        .args(&auth_args)
        .get_matches();

    if app.subcommand().0 == "config" {
        let input: String = terminal::prompt("Enter API Token:");
        let credential = cflare::config::GlobalCredential::Token { api_token: input };
        if let Err(e) = config::save_credential(&credential) {
            terminal::error(format!("{}", e).as_str());
            std::process::exit(1);
        }
        return;
    }

    let api = get_api_client(&app);
    match app.subcommand() {
        ("accounts", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let limit: u32 = cmd.value_of("limit").unwrap_or("50").parse().unwrap();
                accounts::list(&api, 1, limit)
            }
            _ => terminal::error("Unknown command")
        },
        ("zones", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let limit: u32 = cmd.value_of("limit").unwrap_or("50").parse().unwrap();
                zones::list(&api, 1, limit)
            }
            _ => unimplemented!()
        },
        ("cache", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("purge", Some(cmd)) => {
                let all = cmd.is_present("all");
                let zone = resolve_zone(&api, cmd);

                if all {
                    cache::purge_all(&api, zone.as_str());
                } else {
                    let urls = cmd.values_of("url").unwrap();
                    cache::purge_url(&api, zone.as_str(), urls);
                }

            }
            _ => unimplemented!()
        },
        ("dns", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let zone = resolve_zone(&api, cmd);
                let limit: u32 = cmd.value_of("limit").unwrap_or("50").parse().unwrap();
                let wide = cmd.is_present("wide");
                let name = cmd.value_of("name");

                let params = dns::ListParams {
                    zone_id: &zone,
                    page: 1,
                    limit,
                    wide,
                    filters: dns::ListFilters { all: name },
                };
                dns::list(&api, params)
            }
            ("create", Some(cmd)) => {
                let zone = resolve_zone(&api, cmd);
                let content = cmd.value_of("content").unwrap();
                let record_type = cmd.value_of("type").unwrap_or("A");
                let proxied = cmd.is_present("proxied");
                let name = cmd.value_of("name").unwrap();
                let ttl: u32 = cmd.value_of("ttl").unwrap_or("1").parse().unwrap();
                let priority: u16 = cmd.value_of("priority").unwrap_or("0").parse().unwrap();

                let record = dns::CreateParams {
                    zone_id: &zone,
                    name,
                    ttl,
                    proxied,
                    content,
                    record_type,
                    priority,
                };

                dns::create(&api, record)
            }
            ("update", Some(cmd)) => {
                let id = cmd.value_of("id").unwrap();
                let zone_id = resolve_zone(&api, cmd);
                let content = cmd.value_of("content");
                let name = cmd.value_of("name");

                let ttl: Option<u32> = match cmd.value_of("ttl") {
                    Some(val) => {
                        match val.parse() {
                            Ok(ttl) => Some(ttl),
                            Err(_) => None
                        }
                    }
                    None => None,
                };
                let proxied = match cmd.value_of("proxied") {
                    Some(val) => match val {
                        "1" | "true" => Some(true),
                        _ => Some(false)
                    },
                    None => None
                };

                let record = dns::UpdateParams {
                    id,
                    zone_id: &zone_id,
                    name,
                    ttl,
                    proxied,
                    content,
                };

                dns::update(&api, record)
            }
            ("delete", Some(cmd)) => {
                let id: Vec<_> = cmd.values_of("id").unwrap().collect();
                let zone = resolve_zone(&api, cmd);

                dns::delete(&api, &zone, id)
            }
            _ => {}
        },
        _ => unreachable!()
    }
}
