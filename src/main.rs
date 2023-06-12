mod config;
mod ip_fetcher;
mod namesilo;

use namesilo::NamesiloClient;
use config::Config;
use ip_fetcher::IpFetcher;
use log::{info, warn};
use std::{net::Ipv4Addr, process};
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = Config::read()
        .map_err(|err| {
            eprintln!(
                "Unable to read config \"{:}\"",
                Config::config_path().unwrap().to_str().unwrap()
            );
            eprintln!("{err}");
            process::exit(1);
        })
        .unwrap();

    let ip_fetcher = IpFetcher::new(config.ip_fetchers);

    let namesilo_client = NamesiloClient::new(&config.namesilo_api_key, &config.domain);

    let mut interval = time::interval(Duration::from_secs(config.poll_duration_s));

    let mut last_update_ip: Option<Ipv4Addr> = None;
    loop {
        let option_ip = ip_fetcher.get_ip().await;

        if option_ip == None {
            warn!("Unable to resolve current ip")
        }

        if let Some(ip) = option_ip {
            if option_ip != last_update_ip {
                info!("New ip {ip} checking Namesilo...");
                match namesilo_client.set_ipv4(&ip).await {
                    Ok(()) => {
                        info!("Namesilo is updated with latest ip {ip}");
                        last_update_ip = Some(ip);
                    }
                    Err(err) => {
                        warn!("Failed to update DNS records for new ip {ip}");
                        warn!("{err}");
                    },
                };
            }
        }

        interval.tick().await;
    }
}
