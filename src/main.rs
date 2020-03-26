use clap::{AppSettings, Arg, App, SubCommand};
#[allow(unused_imports)]
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    Environment,
    HttpApiClient,
    HttpApiClientConfig,
};
use cloudflare_cli::commands::{
    dns,
    zones,
    accounts,
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const MAX_TTL: u32 = 2_147_483_647;

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
            if value < 1 || value > MAX_TTL {
                return Err(String::from(format!("Value must be between 1 and {} seconds", MAX_TTL)));
            }
            Ok(())
        }
        Err(_) => Err(format!("Value must be an integer; received: {}", arg))
    }
}

fn main() {
    let auth_args = vec![
        Arg::with_name("email")
            .long("email")
            .short("e")
            .help("Email address associated with your account")
            .takes_value(true)
            .env("CF_EMAIL"),
        Arg::with_name("key")
            .long("key")
            .short("k")
            .help("API token generated on the \"My Account\" page")
            .takes_value(true)
            .env("CF_KEY"),
        Arg::with_name("token")
            .long("token")
            .short("t")
            .help("API token generated on the \"My Account\" page")
            .takes_value(true)
            .env("CF_TOKEN"),
    ];
    let limit = Arg::with_name("limit")
        .short("l")
        .long("limit")
        .validator(valid_u32)
        .takes_value(true);

    let commands = vec![
        SubCommand::with_name("config"),
        SubCommand::with_name("accounts")
            .subcommands(vec![
                SubCommand::with_name("list").arg(
                    limit.clone().default_value("50")
                ),
                SubCommand::with_name("describe"),
            ]).setting(AppSettings::ArgRequiredElseHelp),
        SubCommand::with_name("zones")
            .subcommands(vec![
                SubCommand::with_name("list")
                    .arg(limit.clone().default_value("50")),
            ]).setting(AppSettings::ArgRequiredElseHelp),
        SubCommand::with_name("dns")
            .subcommands(vec![
                SubCommand::with_name("list")
                    .arg(Arg::with_name("zone").takes_value(true).required(true))
                    .arg(Arg::with_name("wide").long("wide").short("w"))
                    .arg(Arg::with_name("name").long("name").short("n")
                        .takes_value(true).help("Filter by name. Performs partial matching"))
                    .arg(limit.clone().default_value("50")),
                SubCommand::with_name("describe"),
                SubCommand::with_name("create")
                    .arg(Arg::with_name("name")
                        .takes_value(true)
                        .required(true)
                        .help("DNS record name")
                    )
                    .arg(Arg::with_name("zone")
                        .short("z")
                        .long("zone-id")
                        .takes_value(true)
                        .required(true))
                    .arg(Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .takes_value(true)
                        .required(true)
                        .help("DNS record content")
                    )
                    .arg(Arg::with_name("type")
                        .long("type")
                        .takes_value(true)
                        .possible_values(&["A", "AAAA", "CNAME", "MX", "TXT", "NS"])
                        .default_value("A")
                        .required(true))
                    .arg(Arg::with_name("ttl")
                        .long("ttl")
                        .validator(valid_ttl)
                        .takes_value(true)
                        .default_value("1")
                        .required(true)
                        .help("Time to live for DNS record. Value of 1 is 'automatic'")
                    )
                    .arg(Arg::with_name("proxied")
                        .long("proxied")
                        .validator(valid_priority)
                        .help("Used with some records like MX and SRV to determine priority")
                    )
                    .arg(Arg::with_name("priority")
                        .long("priority")
                        .validator(valid_priority)
                        .takes_value(true)
                        .default_value("0")
                        .help("Used with some records like MX and SRV to determine priority")
                    )
            ]).setting(AppSettings::ArgRequiredElseHelp),
    ];

    let app = App::new("Cloudflare command-line tool")
        .version(VERSION)
        .subcommands(commands)
        .setting(AppSettings::ArgRequiredElseHelp)
        .args(&auth_args)
        .get_matches();

    let email = app.value_of("email");
    let key = app.value_of("key");
    let token = app.value_of("token");

    let credentials: Credentials = if let Some(key) = key {
        Credentials::UserAuthKey {
            email: email.unwrap().to_string(),
            key: key.to_string(),
        }
    } else if let Some(token) = token {
        Credentials::UserAuthToken {
            token: token.to_string(),
        }
    } else {
        panic!("Either API token or API key + email pair must be provided")
    };

    let api = HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    ).unwrap();

    match app.subcommand() {
        ("accounts", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let limit: u32 = cmd.value_of("limit").unwrap().parse().unwrap();
                accounts::list(&api, 1, limit)
            }
            _ => {}
        },
        ("dns", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let zone = cmd.value_of("zone").unwrap();
                let limit: u32 = cmd.value_of("limit").unwrap().parse().unwrap();
                let wide = cmd.is_present("wide");
                let name = cmd.value_of("name");

                let params = dns::ListParams {
                    zone_id: zone,
                    page: 1,
                    limit,
                    wide,
                    filters: dns::ListFilters { all: name },
                };
                dns::list(&api, params)
            }
            ("describe", Some(_)) => {}
            ("create", Some(cmd)) => {
                let zone = cmd.value_of("zone").unwrap();
                let content = cmd.value_of("content").unwrap();
                let record_type = cmd.value_of("type").unwrap();
                let proxied = cmd.is_present("proxied");
                let name = cmd.value_of("name").unwrap();
                let ttl: u32 = cmd.value_of("ttl").unwrap().parse().unwrap();
                let priority: u16 = cmd.value_of("priority").unwrap().parse().unwrap();

                let record = dns::CreateParams {
                    zone_id: zone,
                    name,
                    ttl,
                    proxied,
                    content,
                    record_type,
                    priority,
                };

                dns::create(&api, zone, record)
            }
            _ => {}
        },
        ("zones", Some(sub_cmd)) => match sub_cmd.subcommand() {
            ("list", Some(cmd)) => {
                let limit: u32 = cmd.value_of("limit").unwrap().parse().unwrap();
                zones::list(&api, 1, limit)
            }
            _ => {}
        },
        _ => {} // Either no sub-command or one not tested for...
    }
}
