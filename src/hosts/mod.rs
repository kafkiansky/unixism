use std::{
    error, fmt, fs,
    io::{self, BufRead, BufReader},
    net::{self, AddrParseError},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Host {
    pub ip: net::IpAddr,
    pub names: Vec<String>,
}

impl Host {
    fn new(ip: net::IpAddr, names: Vec<String>) -> Self {
        Self { ip, names }
    }
}

impl FromStr for Host {
    type Err = ParseHostsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line = s.trim().split_whitespace();

        let host = Host::new(
            line.next().unwrap_or_default().parse()?,
            line.map(String::from).collect(),
        );

        Ok(host)
    }
}

#[derive(Debug)]
pub enum ParseHostsError {
    IPAddrParseError(AddrParseError),
    IOError(io::Error),
}

impl error::Error for ParseHostsError {}

impl fmt::Display for ParseHostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IPAddrParseError(error) => write!(f, "{}", error),
            Self::IOError(error) => write!(f, "{}", error),
        }
    }
}

impl From<AddrParseError> for ParseHostsError {
    fn from(value: AddrParseError) -> Self {
        ParseHostsError::IPAddrParseError(value)
    }
}

impl From<io::Error> for ParseHostsError {
    fn from(value: io::Error) -> Self {
        ParseHostsError::IOError(value)
    }
}

///
/// ```no_run
/// let hosts = unixism::hosts::parse(std::fs::File::open("/etc/hosts").unwrap()).unwrap();
///
/// for host in hosts {
///     println!("ip: {}, names: {:#?}", host.ip, host.names);
/// }
/// ```
pub fn parse<R>(reader: R) -> Result<impl Iterator<Item = Host>, ParseHostsError>
where
    R: io::Read,
{
    let hosts = BufReader::new(reader)
        .lines()
        .map(Result::unwrap_or_default)
        .filter(|line| !line.is_empty() && !line.starts_with("#") && !line.starts_with(" "))
        .map(|line| line.parse::<Host>())
        .collect::<Result<Vec<Host>, ParseHostsError>>()?;

    Ok(hosts.into_iter())
}

///
/// Same as parse, but parses the `/etc/hosts` as default.
///
/// ```no_run
/// let hosts = unixism::hosts::parse_default().unwrap();
/// ```
/// ```
pub fn parse_default() -> Result<impl Iterator<Item = Host>, ParseHostsError> {
    parse(fs::File::open("/etc/hosts")?)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn it_parse() {
        let parsed = parse(Cursor::new(
            r#"
127.0.0.1	localhost

# The following lines are desirable for IPv6 capable hosts
::1     ip6-localhost ip6-loopback
fe00::0 ip6-localnet
ff02::1 ip6-allnodes
ff02::2 ip6-allrouters
        "#,
        ));
        assert!(parsed.is_ok());

        let hosts = parsed.unwrap().collect::<Vec<_>>();
        assert_eq!(5, hosts.len());
        assert_eq!(
            vec![
                Host::new("127.0.0.1".parse().unwrap(), vec!["localhost".to_owned()]),
                Host::new(
                    "::1".parse().unwrap(),
                    vec!["ip6-localhost".to_owned(), "ip6-loopback".to_owned()]
                ),
                Host::new("fe00::0".parse().unwrap(), vec!["ip6-localnet".to_owned()]),
                Host::new("ff02::1".parse().unwrap(), vec!["ip6-allnodes".to_owned()]),
                Host::new(
                    "ff02::2".parse().unwrap(),
                    vec!["ip6-allrouters".to_owned()]
                ),
            ],
            hosts
        );
    }
}
