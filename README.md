cloudflare-cli (cflare)
------------

`cflare` is a command-line tool that lets you manage some aspects of your Cloudflare account.

### 🚀 Installation

#### Install with cargo

```shell script
cargo install cloudflare-cli
```

TBD

### 📚 Usage

List available flags with `cf -h`

```
Cloudflare command-line tool
g
USAGE:
    cloudflare-cli [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --email <email>    Email address associated with your account [env: CF_EMAIL=]
    -k, --key <key>        API token generated on the "My Account" page [env: CF_KEY=]
    -t, --token <token>    API token generated on the "My Account" page [env: CF_TOKEN=]

SUBCOMMANDS:
    accounts
    config
    dns
    help        Prints this message or the help of the given subcommand(s)
    zones
```
