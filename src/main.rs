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
    accounts
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn is_u32(arg: String) -> Result<(), String> {
    match arg.parse::<u32>() {
        Ok(_) => Ok(()),
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
        .validator(is_u32)
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

                let params = dns::ListParams{
                    zone_id: zone,
                    page: 1,
                    limit,
                    wide,
                    filters: dns::ListFilters { all: name }
                };
                dns::list(&api, params)
            }
            ("describe", Some(_)) => {}
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
