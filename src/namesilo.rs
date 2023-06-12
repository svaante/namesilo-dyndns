use anyhow::{anyhow, Result};
use log::{debug, info};
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::net::Ipv4Addr;

const NAMESILO_BASE_API_URL: &str = "https://www.namesilo.com/api";

#[derive(Debug, Deserialize, PartialEq)]
pub struct ResourceRecord {
    record_id: String,
    r#type: String,
    value: String,
    host: String,
}

pub struct NamesiloClient {
    key: String,
    domain: String,
}

impl NamesiloClient {
    pub fn new(key: &str, domain: &str) -> Self {
        let client = Self {
            key: key.to_string(),
            domain: domain.to_string(),
        };
        client
    }

    pub async fn set_ipv4(&self, ip: &Ipv4Addr) -> Result<()> {
        let records = self.list_records().await?;
        let ip_str = ip.to_string();

        let mut update_ip_tasks =  Vec::new();
        for record in &records {
            if record.r#type == "A" && record.value != ip_str {
                update_ip_tasks.push(
                    self.update_record_value(&record.record_id, &ip_str)
                );
            }
        }

        if update_ip_tasks.is_empty() {
            info!("No record to update for {:}", self.domain);
        }

        // Wait for all update tasks to complete
        for task in update_ip_tasks {
            task.await?;
        }

        Ok(())
    }

    fn create_url(&self, endpoint: &str, query: Vec<(&str, &str)>) -> String {
        format!(
            "{NAMESILO_BASE_API_URL}/{:}?version=1&type=xml&key={:}&domain={:}{:}",
            endpoint,
            self.key,
            self.domain,
            query
                .into_iter()
                .fold(String::new(), |acc, (key, value)| acc
                    + format!("&{key}={value}").as_str())
        )
    }

    async fn list_records(&self) -> Result<Vec<ResourceRecord>> {
        let url = self.create_url("dnsListRecords", vec![]);
        let text = reqwest::get(url).await?.text().await?;

        #[derive(Debug, Deserialize, PartialEq)]
        struct Namesilo {
            reply: Reply,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Reply {
            code: u32,
            detail: String,
            resource_record: Vec<ResourceRecord>,
        }

        debug!("dnsListRecords response {text}");

        let namesilo_reply: Namesilo = from_str(&text)?;

        if namesilo_reply.reply.code != 300 {
            return Err(anyhow!(
                "Unable to list records; code={:}",
                namesilo_reply.reply.code
            ));
        }

        Ok(namesilo_reply.reply.resource_record)
    }

    async fn update_record_value(&self, resource_id: &str, value: &str) -> Result<()> {
        let query = vec![("rrid", resource_id), ("rrvalue", value)];

        let url = self.create_url("dnsUpdateRecord", query);
        let text = reqwest::get(url).await?.text().await?;

        #[derive(Debug, Deserialize, PartialEq)]
        struct Namesilo {
            reply: Reply,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Reply {
            code: u32,
            detail: String,
        }

        debug!("dnsUpdateRecord response {text}");

        let namesilo_reply: Namesilo = from_str(&text)?;

        if namesilo_reply.reply.code != 300 {
            return Err(anyhow!(
                "Unable to update record; code={:} detail=\"{:}\"",
                namesilo_reply.reply.code,
                namesilo_reply.reply.detail
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_params() {
        let client = NamesiloClient::new("12345", "namesilo.com");

        let list_current_record = client.create_url("dnsListRecords", vec![]);
        assert_eq!(
            list_current_record,
            "https://www.namesilo.com/api/dnsListRecords?version=1&type=xml&key=12345&domain=namesilo.com"
        );

        let dns_update_record = client.create_url(
            "dnsUpdateRecord",
            vec![
                ("rrid", "1a2b3"),
                ("rrhost", "test"),
                ("rrvalue", "55.55.55.55"),
                ("rrttl", "7207"),
            ],
        );
        assert_eq!(
            dns_update_record,
            "https://www.namesilo.com/api/dnsUpdateRecord?version=1&type=xml&key=12345&domain=namesilo.com&rrid=1a2b3&rrhost=test&rrvalue=55.55.55.55&rrttl=7207".to_string()
        );

        let add_dns_record = client.create_url(
            "dnsAddRecord",
            vec![
                ("rrtype", "A"),
                ("rrhost", "test"),
                ("rrvalue", "55.55.55.55"),
                ("rrttl", "7207"),
            ],
        );
        assert_eq!(
            add_dns_record,
            "https://www.namesilo.com/api/dnsAddRecord?version=1&type=xml&key=12345&domain=namesilo.com&rrtype=A&rrhost=test&rrvalue=55.55.55.55&rrttl=7207"
        );
    }
}
