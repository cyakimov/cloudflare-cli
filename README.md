cloudflare-cli (cflare)
------------

`cflare` is a command-line tool that lets you manage some aspects of your Cloudflare account.

### 🚀 Installation

#### Install with Homebrew

```shell script
brew install cyakimov/tools/cflare
```

#### Install with cargo

```shell script
cargo install cflare
```

### 🏃‍♂️ Quickstart

1. Create a [Cloudflare API token](https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys)
2. Run `cflare config` & paste the API token

You're all set now.

### 📚 Usage

List available commands with `cflare -h`

Examples:

```shell script
cflare accounts list
cflare zones list
cflare dns list --zone mydomain.com
cflare dns create --zone mydomain.com -c 1.1.1.1 mysubdomain --ttl 3600
cflare cache purge --zone mydomain.com -u https://mydomain.com/css/styles.css https://mydomain.com/js/main.js ...
cflare cache purge --zone mydomain.com --all
```

**Overriding config file credentials:**

Providing any of the `--email`, `--key` or `--token` arguments overrides the config file.

### Future plan

* Improve error formatting.
* Context switching _a la `kubectl`_. Useful when you manage multiple Cloudflare accounts. 
* ~~Add support for `cache` command to purge the cache.~~
* Add support for `pagerules` command to manage Page rules.
* Add support for `certificates` command to manage Origin certificates.
