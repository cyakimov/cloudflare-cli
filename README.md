cloudflare-cli (cflare)
------------

`cflare` is a command-line tool that lets you manage some aspects of your Cloudflare account.

### 🚀 Installation

#### Install with cargo

```shell script
cargo install cflare
```

### 📚 Usage

List available commands with `cflare -h`

```
Cloudflare command-line tool
g
USAGE:
    cloudflare-cli [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --email <email>    Email address associated with your account
    -k, --key <key>        API key generated on the "My Account" page
    -t, --token <token>    API token generated on the "My Account" page

SUBCOMMANDS:
    accounts
    config
    dns
    zones
    help
```
