use crate::cmd::aliasremove::get_alias;
use crate::cmd::lsmain;
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::Config;
use chrono::DateTime;
use chrono::Local;
use human_bytes::human_bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

const PRINT_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %Z";

// Represents content message for S3 object details
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ContentMessage {
    status: String,
    filetype: String,
    time: DateTime<Local>,
    size: i64,
    key: String,
    etag: String,
    url: Option<String>,

    version_id: Option<String>,
    version_ord: Option<i32>,
    version_index: Option<i32>,
    is_delete_marker: Option<bool>,
    storage_class: Option<String>,

    metadata: HashMap<String, String>,
    tags: HashMap<String, String>,
    is_directory: bool,
}

impl ContentMessage {
    fn new() -> Self {
        Self {
            status: "success".to_string(),
            filetype: "file".to_string(),
            time: Local::now(),
            size: 0,
            key: "".to_string(),
            etag: "".to_string(),
            url: None,
            version_id: None,
            version_ord: None,
            version_index: None,
            is_delete_marker: None,
            storage_class: None,
            metadata: HashMap::new(),
            tags: HashMap::new(),
            is_directory: false,
        }
    }

    fn to_string(&self) -> String {
        format!(
            "[{}] {:>10} {}",
            self.time.format(PRINT_DATE_FORMAT),
            //humantime::format_rf (self.time.into()),
            human_bytes(self.size as f64),
            self.key
        )
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

// Wrapper for the AWS S3 client
struct S3ClientWrapper {
    client: S3Client,
    bucket: String,
}

#[async_trait]
trait Client {
    async fn list(&self, options: &lsmain::LsOptions) -> mpsc::Receiver<ContentMessage>;
    fn get_url(&self) -> String;
}

enum ListResult {
    Objects(aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output), // 假设 `ListObjectsV2Output` 是 list_objects_v2 的返回类型
    Buckets(aws_sdk_s3::operation::list_buckets::ListBucketsOutput), // 假设 `ListBucketsOutput` 是 list_buckets 的返回类型
}

#[async_trait]
impl Client for S3ClientWrapper {
    async fn list(&self, options: &lsmain::LsOptions) -> mpsc::Receiver<ContentMessage> {
        let (tx, rx) = mpsc::channel(100);
        let client = self.client.clone();
        let bucket = self.bucket.clone();

        let delimiter = if options.recursive { "" } else { "/" };

        tokio::spawn(async move {
            let mut continuation_token = None;

            loop {
                let result: Result<ListResult> = if !bucket.is_empty() {
                    client
                        .list_objects_v2()
                        .bucket(&bucket)
                        .set_continuation_token(continuation_token.clone())
                        .delimiter(delimiter)
                        .send()
                        .await
                        .map(ListResult::Objects)
                        .map_err(anyhow::Error::from) // 使用 `anyhow` 将错误包装
                } else {
                    client
                        .list_buckets()
                        .send()
                        .await
                        .map(ListResult::Buckets)
                        .map_err(anyhow::Error::from) // 使用 `anyhow` 将错误包装
                };

                match result {
                    Ok(output) => {
                        match output {
                            ListResult::Buckets(ret) => {
                                //if ret.buckets() {
                                for bucket in ret.buckets() {
                                    let _ = tx
                                        .send(ContentMessage {
                                            key: bucket.name().unwrap_or_default().to_string(), // bucket 名称
                                            size: 0,            // bucket 没有大小
                                            is_directory: true, // 标识为目录
                                            ..ContentMessage::new()
                                        })
                                        .await;
                                }
                                //}
                                // ListBuckets 不支持分页，因此直接退出
                                break;
                            }
                            ListResult::Objects(objs) => {
                                if !objs.common_prefixes().is_empty() {
                                    for prefix in objs.common_prefixes() {
                                        let _ = tx
                                            .send(ContentMessage {
                                                key: prefix
                                                    .prefix()
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                size: 0,            // 目录没有大小
                                                is_directory: true, // 自定义字段标识为目录
                                                ..ContentMessage::new()
                                            })
                                            .await;
                                    }
                                }

                                // 处理文件 (contents)
                                if !objs.contents().is_empty() {
                                    for object in objs.contents() {
                                        let _ = tx
                                            .send(ContentMessage {
                                                key: object.key().unwrap_or_default().to_string(),
                                                size: object.size().unwrap(),
                                                storage_class: object
                                                    .storage_class()
                                                    .map(|sc| sc.as_str().to_string()),
                                                is_directory: false, // 自定义字段标识为文件
                                                ..ContentMessage::new()
                                            })
                                            .await;
                                    }
                                }
                                // 更新 continuation token 并检查是否需要继续
                                continuation_token = objs.next_continuation_token;
                                if continuation_token.is_none() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("Error listing objects: {:?}", error);
                        let _ = tx
                            .send(ContentMessage {
                                key: "".to_string(),
                                size: 0,
                                ..ContentMessage::new()
                            })
                            .await;
                        break;
                    }
                }
            }
        });

        rx
    }

    fn get_url(&self) -> String {
        format!("s3://{}", self.bucket)
    }
}

async fn do_list(client: impl Client + Send + Sync, options: &lsmain::LsOptions) {
    let mut last_path = String::new();
    let mut total_size = 0;
    let mut total_objects = 0;

    let mut receiver = client.list(options).await;
    while let Some(content) = receiver.recv().await {
        total_size += content.size;
        total_objects += 1;

        if last_path != content.key {
            println!("{}", content.to_string());
            last_path = content.key.clone();
        }
    }

    println!("Total Size: {} bytes", total_size);
    println!("Total Objects: {}", total_objects);
}

//#[tokio::main]
fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

pub async fn ls(opt: &lsmain::LsOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opt.path.is_empty() {
        println!("path is empty");
        return Err("Path is empty".into());
    }
    let (alias, key) = split_first_part(&opt.path);

    //let ret = get_alias(alias);
    if let Ok(conf) = get_alias(alias) {
        println!("get config {} suc", alias);
        let credentials = Credentials::new(conf.access_key, conf.secret_key, None, None, "");
        let region = Region::new("us-east-1".to_string());

        let config = Config::builder()
            .region(region)
            .endpoint_url(conf.url.to_string())
            .credentials_provider(credentials)
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();

        let s3_client = S3Client::from_conf(config);
        //let bucket = "xxxxx".to_string();

        let (bucket, key) = split_first_part(key);

        println!("bucket is {}", bucket);
        println!("key is {}", key);

        let client = S3ClientWrapper {
            client: s3_client,
            bucket: bucket.to_string(),
        };

        do_list(client, opt).await;
        println!("------------------");

        Ok(())
    } else {
        println!("path is empty");
        return Err("alias is null".into());
    }

    // let s3_client = S3Client::from_conf(config);
    // let bucket = "xxxxx".to_string();

    // let client = S3ClientWrapper {
    //     client: s3_client,
    //     bucket,
    // };

    // do_list(client, opt).await;
    // println!("------------------");

    // Ok(())
}
