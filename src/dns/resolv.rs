use std::{
    error, fmt, fs,
    io::{self, BufRead, BufReader},
    net::{self, AddrParseError},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug)]
pub enum ParseConfigError {
    UnknownOption(String),
    IPAddrParseError(AddrParseError),
    ParseIntError(ParseIntError),
    IOError(io::Error),
}

impl error::Error for ParseConfigError {}

impl fmt::Display for ParseConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption(unknown) => write!(f, "unknown option parsed: {}", unknown),
            Self::IPAddrParseError(error) => write!(f, "{}", error),
            Self::ParseIntError(error) => write!(f, "{}", error),
            Self::IOError(error) => write!(f, "{}", error),
        }
    }
}

impl From<AddrParseError> for ParseConfigError {
    fn from(value: AddrParseError) -> Self {
        ParseConfigError::IPAddrParseError(value)
    }
}

impl From<ParseIntError> for ParseConfigError {
    fn from(value: ParseIntError) -> Self {
        ParseConfigError::ParseIntError(value)
    }
}

impl From<io::Error> for ParseConfigError {
    fn from(value: io::Error) -> Self {
        ParseConfigError::IOError(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IPPair(pub net::IpAddr, pub Option<net::IpAddr>);

impl FromStr for IPPair {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pair = s.split("/");

        Ok(Self(
            pair.next().unwrap_or_default().parse::<net::IpAddr>()?,
            if let Some(netmask) = pair.next() {
                Some(netmask.parse::<net::IpAddr>()?)
            } else {
                None
            },
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigOption {
    DEBUG,
    NDots(usize),
    Timeout(usize),
    Attempts(usize),
    ROTATE,
    NOAAAA,
    NOCHECKNAME,
    INET6,
    IP6BSTRING,
    IP6DOTINT,
    NOIP6DOTINT,
    EDNS0,
    SNGLKUP,
    SNGLKUPREOP,
    NOTLDQUERY,
    USEVC,
    NORELOAD,
    TRUSTAD,
}

impl FromStr for ConfigOption {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pair = s.split(":");

        match pair.next().unwrap_or_default() {
            "debug" => Ok(ConfigOption::DEBUG),
            "rotate" => Ok(ConfigOption::ROTATE),
            "no-aaaa" => Ok(ConfigOption::NOAAAA),
            "no-check-names" => Ok(ConfigOption::NOCHECKNAME),
            "inet6" => Ok(ConfigOption::INET6),
            "ip6-bytestring" => Ok(ConfigOption::IP6BSTRING),
            "ip6-dotint" => Ok(ConfigOption::IP6DOTINT),
            "no-ip6-dotint" => Ok(ConfigOption::NOIP6DOTINT),
            "edns0" => Ok(ConfigOption::EDNS0),
            "single-request" => Ok(ConfigOption::SNGLKUP),
            "single-request-reopen" => Ok(ConfigOption::SNGLKUPREOP),
            "no-tld-query" => Ok(ConfigOption::NOTLDQUERY),
            "use-vc" => Ok(ConfigOption::USEVC),
            "no-reload" => Ok(ConfigOption::NORELOAD),
            "trust-ad" => Ok(ConfigOption::TRUSTAD),
            option @ ("ndots" | "timeout" | "attempts") => {
                let number = pair.next().unwrap_or_default().parse::<usize>()?;

                Ok(match option {
                    "ndots" => ConfigOption::NDots(number),
                    "timeout" => ConfigOption::Timeout(number),
                    "attempts" => ConfigOption::Attempts(number),
                    _ => unreachable!("option {option} is not covered"),
                })
            }
            unknown => Err(ParseConfigError::UnknownOption(unknown.to_owned())),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub nameservers: Vec<net::IpAddr>,
    pub search_domains: Vec<String>,
    pub sort_list: Vec<IPPair>,
    pub options: Vec<ConfigOption>,
}

impl Config {
    fn from_items(items: Vec<ConfigItem>) -> Self {
        let mut config = Self::default();

        for item in items {
            match item {
                ConfigItem::Nameserver(nameserver) => config.nameservers.push(nameserver),
                ConfigItem::SearchDomains(domains) => config.search_domains.extend(domains),
                ConfigItem::Domain(domain) => config.search_domains.push(domain),
                ConfigItem::SortList(lists) => config.sort_list.extend(lists),
                ConfigItem::Options(options) => config.options.extend(options),
            }
        }

        config
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigItem {
    Nameserver(net::IpAddr),
    Domain(String),
    SearchDomains(Vec<String>),
    SortList(Vec<IPPair>),
    Options(Vec<ConfigOption>),
}

impl FromStr for ConfigItem {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("nameserver") => Ok(ConfigItem::Nameserver(
                s.split_once("nameserver")
                    .unwrap_or_default()
                    .1
                    .trim()
                    .parse::<net::IpAddr>()?,
            )),
            s if s.starts_with("domain") => Ok(ConfigItem::Domain(
                s.split_once("domain")
                    .unwrap_or_default()
                    .1
                    .trim()
                    .to_owned(),
            )),
            s if s.starts_with("search") => Ok(ConfigItem::SearchDomains(
                s.split_once("search")
                    .unwrap_or_default()
                    .1
                    .trim()
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>(),
            )),
            s if s.starts_with("sortlist") => Ok(ConfigItem::SortList(
                s.split_once("sortlist")
                    .unwrap_or_default()
                    .1
                    .trim()
                    .split_whitespace()
                    .map(|line| line.parse::<IPPair>())
                    .collect::<Result<Vec<IPPair>, ParseConfigError>>()?,
            )),
            s if s.starts_with("options") => Ok(ConfigItem::Options(
                s.split_once("options")
                    .unwrap_or_default()
                    .1
                    .trim()
                    .split_whitespace()
                    .map(|line| line.parse::<ConfigOption>())
                    .collect::<Result<Vec<ConfigOption>, ParseConfigError>>()?,
            )),
            unknown => Err(ParseConfigError::UnknownOption(unknown.to_owned())),
        }
    }
}

///
/// ```no_run
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let config = unixism::dns::resolv::parse(std::fs::File::open("/etc/resolv.conf").unwrap()).unwrap();
///
/// for nameserver in config.nameservers {
///     println!("{}", nameserver.to_string());
/// }
///
/// for domain in config.search_domains {
///     println!("{}", domain);
/// }
///
/// for pair in config.sort_list {
///     println!(
///         "ip: {}, mask: {}",
///          pair.0.to_string(),
///          pair.1
///              .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
///              .to_string()
///     );
/// }
///
/// for option in config.options {
///     match option {
///         unixism::dns::resolv::ConfigOption::Attempts(attempts) => println!("attempts: {attempts}"),
///         _ => {}
///     }
/// }
/// ```
pub fn parse<R>(reader: R) -> Result<Config, ParseConfigError>
where
    R: io::Read,
{
    let items = BufReader::new(reader)
        .lines()
        .map(Result::unwrap_or_default)
        .filter(|line| !line.is_empty() && !line.starts_with("#") && !line.starts_with(" "))
        .map(|line| line.parse::<ConfigItem>())
        .collect::<Result<Vec<ConfigItem>, ParseConfigError>>()?;

    Ok(Config::from_items(items))
}

///
/// Same as parse, but parses the `/etc/resolv.conf` as default.
///
/// ```no_run
/// let config = unixism::dns::resolv::parse_default().unwrap();
/// ```
pub fn parse_default() -> Result<Config, ParseConfigError> {
    parse(fs::File::open("/etc/resolv.conf")?)
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, net::IpAddr, vec};

    use super::*;

    #[test]
    fn it_parse() {
        let config = parse(Cursor::new(
            r#"
nameserver 127.0.0.53
nameserver 127.0.0.52
options edns0 trust-ad timeout:5 attempts:2 ndots:3 debug
search . 127.0.0.1
sortlist 130.155.160.0/255.255.240.0 130.155.0.0
        "#,
        ));
        assert!(config.is_ok());

        let Config {
            nameservers,
            search_domains,
            sort_list,
            options,
        } = config.unwrap();

        assert_eq!(2, nameservers.len());
        assert_eq!(
            vec![
                "127.0.0.53".parse::<IpAddr>().unwrap(),
                "127.0.0.52".parse::<IpAddr>().unwrap(),
            ],
            nameservers
        );

        assert_eq!(2, search_domains.len());
        assert_eq!(vec![".".to_owned(), "127.0.0.1".to_owned()], search_domains);

        assert_eq!(2, sort_list.len());
        assert_eq!(
            vec![
                IPPair(
                    "130.155.160.0".parse().unwrap(),
                    Some("255.255.240.0".parse().unwrap())
                ),
                IPPair("130.155.0.0".parse().unwrap(), None)
            ],
            sort_list
        );

        assert_eq!(6, options.len());
        assert_eq!(
            vec![
                ConfigOption::EDNS0,
                ConfigOption::TRUSTAD,
                ConfigOption::Timeout(5),
                ConfigOption::Attempts(2),
                ConfigOption::NDots(3),
                ConfigOption::DEBUG,
            ],
            options
        );
    }
}
