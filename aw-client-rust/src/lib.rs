extern crate gethostname;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate aw_models;
extern crate serde_json;

use std::collections::HashMap;
use std::vec::Vec;

use chrono::{DateTime, Utc};
use serde_json::Map;

pub use aw_models::{Bucket, BucketMetadata, Event};

#[derive(Deserialize)]
pub struct Info {
    pub hostname: String,
    pub testing: bool,
}

#[derive(Debug)]
pub struct AwClient {
    client: reqwest::blocking::Client,
    pub baseurl: String,
    pub name: String,
    pub hostname: String,
}

impl AwClient {
    pub fn new(ip: &str, port: &str, name: &str) -> AwClient {
        let baseurl = format!("http://{}:{}", ip, port);
        let client = reqwest::blocking::Client::new();
        let hostname = gethostname::gethostname().into_string().unwrap();
        AwClient {
            client,
            baseurl,
            name: name.to_string(),
            hostname,
        }
    }

    pub fn get_bucket(&self, bucketname: &str) -> Result<Bucket, reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}", self.baseurl, bucketname);
        let bucket: Bucket = self.client.get(&url).send()?.json()?;
        Ok(bucket)
    }

    pub fn get_buckets(&self) -> Result<HashMap<String, Bucket>, reqwest::Error> {
        let url = format!("{}/api/0/buckets/", self.baseurl);
        Ok(self.client.get(&url).send()?.json()?)
    }

    pub fn create_bucket(&self, bucketname: &str, buckettype: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}", self.baseurl, bucketname);
        let data = Bucket {
            bid: None,
            id: bucketname.to_string(),
            client: self.name.clone(),
            _type: buckettype.to_string(),
            hostname: self.hostname.clone(),
            data: Map::default(),
            metadata: BucketMetadata::default(),
            events: None,
            created: None,
            last_updated: None,
        };
        self.client.post(&url).json(&data).send()?;
        Ok(())
    }

    pub fn delete_bucket(&self, bucketname: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}", self.baseurl, bucketname);
        self.client.delete(&url).send()?;
        Ok(())
    }

    pub fn get_events(
        &self,
        bucketname: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Event>, reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}/events", self.baseurl, bucketname);

        let mut req = self.client.get(&url);
        if let Some(start) = start {
            req = req.query(&[("start", start.to_rfc3339())])
        }
        if let Some(end) = end {
            req = req.query(&[("end", end.to_rfc3339())]);
        }
        if let Some(limit) = limit {
            req = req.query(&[("limit", limit)]);
        }

        Ok(req.send()?.json()?)
    }

    pub fn insert_event(&self, bucketname: &str, event: &Event) -> Result<(), reqwest::Error> {
        let mut eventlist = Vec::new();
        eventlist.push(event.clone());
        self.insert_events(bucketname, eventlist)
    }

    pub fn insert_events(
        &self,
        bucketname: &str,
        events: Vec<Event>,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}/events", self.baseurl, bucketname);
        self.client.post(&url).json(&events).send()?;
        Ok(())
    }

    pub fn heartbeat(
        &self,
        bucketname: &str,
        event: &Event,
        pulsetime: f64,
    ) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/api/0/buckets/{}/heartbeat?pulsetime={}",
            self.baseurl, bucketname, pulsetime
        );
        self.client.post(&url).json(&event).send()?;
        Ok(())
    }

    pub fn delete_event(&self, bucketname: &str, event_id: i64) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/api/0/buckets/{}/events/{}",
            self.baseurl, bucketname, event_id
        );
        self.client.delete(&url).send()?;
        Ok(())
    }

    pub fn get_event_count(
        &self,
        bucketname: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<i64, reqwest::Error> {
        let url = format!("{}/api/0/buckets/{}/events/count", self.baseurl, bucketname);
        let mut req = self.client.get(&url);

        if let Some(start) = start {
            req = req.query(&[("start", start.to_rfc3339())])
        }
        if let Some(end) = end {
            req = req.query(&[("end", end.to_rfc3339())]);
        }

        let res = req.send()?.text()?;
        let count: i64 = match res.parse() {
            Ok(count) => count,
            Err(err) => panic!("could not parse get_event_count response: {:?}", err),
        };
        Ok(count)
    }

    pub fn get_info(&self) -> Result<Info, reqwest::Error> {
        let url = format!("{}/api/0/info", self.baseurl);
        Ok(self.client.get(&url).send()?.json()?)
    }
}
