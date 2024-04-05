## Work with linux from Rust.

[![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg?style=flat-square)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/unixism.svg)](https://crates.io/crates/unixism)

## Installation

```bash
cargo add unixism
```

## Contents
- [resolv.conf](#resolv-conf)
- [hosts](#hosts)

### resolv.conf

Parsing an `/etc/resolv.conf` file.

```rust
use unixism::dns;

fn main() {
    let config = dns::resolv::parse_default().unwrap();

    for nameserver in config.nameservers {
        println!("{}", nameserver.to_string());
    }

    for domain in config.search_domains {
        println!("{domain}");
    }

    for option in config.options {
        match option {
            dns::resolv::ConfigOption::Timeout(timeout) => {
                println!("timeout: {timeout}");
            }
            _ => {}
        }
    }
}
```

Parsing any kind of `io::Read` type.

```rust
use std::fs;
use unixism::dns;

fn main() {
    let config = dns::resolv::parse(fs::File::open("local.conf").unwrap()).unwrap();

    for nameserver in config.nameservers {
        println!("{}", nameserver.to_string());
    }

    for domain in config.search_domains {
        println!("{domain}");
    }

    for option in config.options {
        match option {
            dns::resolv::ConfigOption::Timeout(timeout) => {
                println!("timeout: {timeout}");
            }
            _ => {}
        }
    }
}
```

### hosts

Parsing an `/etc/hosts` file.

```rust
use unixism::hosts;

fn main() {
    for host in hosts::parse_default().unwrap() {
        println!("ip: {}, names: {:#?}", host.ip, host.names);
    }
}
```

Parsing any kind of `io::Read` type.

```rust
use std::fs;
use unixism::hosts;

fn main() {
    for host in hosts::parse(fs::File::open("local.conf").unwrap()).unwrap() {
        println!("ip: {}, names: {:#?}", host.ip, host.names);
    }
}
```