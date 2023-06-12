use std::collections::HashMap;
use std::{error::Error, net::Ipv4Addr, str::FromStr};
use anyhow::Result;
use futures::future::join_all;
use log::warn;
use regex::Regex;


const IP_REGEX: &str = r"(?:[0-9]{1,3}\.){3}[0-9]{1,3}";

pub struct IpFetcher {
    ip_services: Vec<String>,
}

impl IpFetcher {
    pub fn new(ip_services: Vec<String>) -> Self {
        Self { ip_services }
    }

    fn text_to_ipv4(text: &String) -> Result<Ipv4Addr, Box<dyn Error>> {
        let regex = Regex::new(IP_REGEX)?;

        let mat = regex.find(&text).ok_or("Unable to get ip from response")?;
        let ipv4_string = text[mat.start()..mat.end()].to_string();
        Ipv4Addr::from_str(&ipv4_string).map_err(|err| err.into())
    }

    async fn get_text(url: &String) -> Result<String, Box<dyn Error>> {
        reqwest::get(url).await?.text().await.map_err(Box::from)
    }

    pub async fn get_ip(&self) -> Option<Ipv4Addr> {
        let requests = join_all(self.ip_services.iter().map(Self::get_text)).await;

        let ips = requests
            .iter()
            .filter_map(|res| {
                res.as_ref()
                    .map_err(|err| {
                        warn!("Failed fetching ip {err}");
                        err
                    })
                    .ok()
            })
            .filter_map(|text| Self::text_to_ipv4(&text).ok());

        let mut counter = HashMap::<Ipv4Addr, usize>::new();

        for ip in ips {
            *counter.entry(ip).or_insert(0) += 1;
        }

        counter
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(item, _)| item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_length() {
        let ip_fetcher = IpFetcher::new(vec!["https://httpbin.org/ip".to_string()]);
        let ip = ip_fetcher.get_ip().await;

        assert!(ip.is_some());
        assert_eq!(ip.unwrap(), Ipv4Addr::new(85, 228, 205, 114));
    }
}
