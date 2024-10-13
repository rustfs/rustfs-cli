use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use chrono::{Local, Utc};
use clap;
use human_bytes::human_bytes;
use indicatif::HumanBytes;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use unicode_normalization::UnicodeNormalization;
//use crate::s3::client as Client;

const PRINT_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %Z";
use glob::Pattern;
use tokio::sync::mpsc;

use crate::s3;

pub struct LsOptions {
    pub path: Option<String>,
    pub rewind: Option<String>,
    pub versions: bool,
    pub recursive: bool,
    pub incomplete: bool,
    pub with_older_versions: bool,
    pub with_delete_markers: bool,
    pub show_dir: String,
    pub with_metadata: bool,
    /* … */
}
struct S3ClientWrapper {
    client: S3Client,
    bucket: String,
}

#[async_trait]
trait Client {
    async fn list(&self, options: &LsOptions) -> mpsc::Receiver<ContentMessage>;
    fn get_url(&self) -> String;
}

enum ListResult {
    Objects(aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output), // 假设 `ListObjectsV2Output` 是 list_objects_v2 的返回类型
    Buckets(aws_sdk_s3::operation::list_buckets::ListBucketsOutput), // 假设 `ListBucketsOutput` 是 list_buckets 的返回类型
}
#[async_trait]
impl Client for S3ClientWrapper {
    async fn list(&self, options: &LsOptions) -> mpsc::Receiver<ContentMessage> {
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
                                    println!("-------------");
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

#[derive(Serialize, Deserialize, Debug)]
struct ContentMessage {
    status: String,
    #[serde(rename = "type")]
    filetype: String,
    #[serde(rename = "lastModified")]
    time: chrono::DateTime<Utc>,
    size: i64,
    key: String,
    etag: String,
    url: Option<String>,

    #[serde(rename = "versionId")]
    version_id: Option<String>,
    #[serde(rename = "versionOrdinal")]
    version_ord: Option<i32>,
    #[serde(rename = "versionIndex")]
    version_index: Option<i32>,
    #[serde(rename = "isDeleteMarker")]
    is_delete_marker: Option<bool>,
    #[serde(rename = "storageClass")]
    storage_class: Option<String>,

    metadata: Option<HashMap<String, String>>,
    tags: Option<HashMap<String, String>>,
    is_directory: bool,
}
impl ContentMessage {
    fn new() -> Self {
        Self {
            status: "success".to_string(),
            filetype: "file".to_string(),
            time: Utc::now(),
            size: 0,
            key: "".to_string(),
            etag: "".to_string(),
            url: None,
            version_id: None,
            version_ord: None,
            version_index: None,
            is_delete_marker: None,
            storage_class: None,
            metadata: Some(HashMap::new()),
            tags: Some(HashMap::new()),
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
const PRINT_DATE: &str = "%Y-%m-%d %H:%M:%S %Z";
impl std::fmt::Display for ContentMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let time_str = self.time.format(PRINT_DATE).to_string();
        let size_str = human_bytes::human_bytes(self.size as f64); // Custom human-readable bytes function
        let mut message = format!("[{}] {}", time_str, size_str);

        if !self.storage_class.is_some() {
            message.push_str(&format!(" {}", self.storage_class.as_ref().unwrap()));
        }

        let mut file_desc = String::new();
        if !self.version_id.is_some() {
            file_desc.push_str(&format!(
                " {} v{}",
                self.version_id.as_ref().unwrap(),
                self.version_ord.unwrap_or(0)
            ));
            file_desc.push_str(if self.is_delete_marker.unwrap() {
                " DEL"
            } else {
                " PUT"
            });
        }

        file_desc.push_str(&format!(" {}", self.key));
        message.push_str(&if self.filetype == "folder" {
            " Dir"
        } else {
            " File"
        });
        write!(f, "{}", message)
    }
}
#[derive(clap::Args, Debug)]
pub struct FindOptions {
    #[arg(help = "alias/bucket (e.g., rustfs/bucketxyz)")]
    pub path: String,

    #[arg(
        long,
        help = "Spawn an external process for each matching object (see FORMAT)"
    )]
    pub exec: Option<String>,

    #[arg(long, help = "Exclude objects matching the wildcard pattern")]
    pub ignore: Option<String>,

    #[arg(long, help = "Include all object versions")]
    pub versions: bool,

    #[arg(long, help = "Find object names matching wildcard pattern")]
    pub name: Option<String>,

    #[arg(
        long,
        help = "Match all objects newer than value in duration string (e.g., 7d10h31s)"
    )]
    pub newer_than: Option<String>,

    #[arg(
        long,
        help = "Match all objects older than value in duration string (e.g., 7d10h31s)"
    )]
    pub older_than: Option<String>,

    #[arg(long, help = "Match directory names matching wildcard pattern")]
    pub path_match: Option<String>,

    #[arg(long, help = "Print in custom format to STDOUT (see FORMAT)")]
    pub print: Option<String>,

    #[arg(long, help = "Match directory and object name with RE2 regex pattern")]
    pub regex: Option<String>,

    #[arg(
        long,
        help = "Match all objects larger than specified size in units (see UNITS)"
    )]
    pub larger: Option<String>,

    #[arg(
        long,
        help = "Match all objects smaller than specified size in units (see UNITS)"
    )]
    pub smaller: Option<String>,

    #[arg(
        long,
        help = "Limit directory navigation to specified depth (default: 0)"
    )]
    pub maxdepth: Option<u32>,

    #[arg(long, help = "Monitor a specified path for newly created object(s)")]
    pub watch: bool,

    #[arg(
        long,
        help = "Match metadata with RE2 regex pattern. Specify each with key=regex. MinIO server only."
    )]
    pub metadata: Option<String>,

    #[arg(
        long,
        help = "Match tags with RE2 regex pattern. Specify each with key=regex. MinIO server only."
    )]
    pub tags: Option<String>,

    #[arg(
        long,
        short = 'C',
        help = "Path to configuration folder (default: \"/home/ldy/.mc\")"
    )]
    pub config_dir: Option<String>,

    #[arg(long, short = 'q', help = "Disable progress bar display")]
    pub quiet: bool,

    #[arg(long, help = "Disable mc internal pager and print to raw stdout")]
    pub disable_pager: bool,

    #[arg(long, help = "Disable color theme")]
    pub no_color: bool,

    #[arg(long, help = "Enable JSON lines formatted output")]
    pub json: bool,

    #[arg(long, help = "Enable debug output")]
    pub debug: bool,

    #[arg(
        long,
        help = "Resolves HOST[:PORT] to an IP address. Example: minio.local:9000=10.10.75.1"
    )]
    pub resolve: Option<String>,

    #[arg(long, help = "Disable SSL certificate verification")]
    pub insecure: bool,

    #[arg(
        long,
        help = "Limits uploads to a maximum rate in KiB/s, MiB/s, GiB/s. (default: unlimited)"
    )]
    pub limit_upload: Option<String>,

    #[arg(
        long,
        help = "Limits downloads to a maximum rate in KiB/s, MiB/s, GiB/s. (default: unlimited)"
    )]
    pub limit_download: Option<String>,
}
pub async fn handle_find_command(opt: &FindOptions) {
    //let _ = find(opt).await;
    let small: u64 = if opt.smaller.is_some() {
        match bytesize::ByteSize::from_str(&opt.smaller.as_ref().unwrap()) {
            Ok(byte_size) => byte_size.as_u64(), // 输出字节数
            Err(e) => 0,
        }
    } else {
        0
    };
    let large: u64 = if opt.larger.is_some() {
        match bytesize::ByteSize::from_str(&opt.larger.as_ref().unwrap()) {
            Ok(byte_size) => byte_size.as_u64(), // 输出字节数
            Err(e) => 0,
        }
    } else {
        0
    };
    let s3_client = s3::client::get_s3client_from_alias("minio").unwrap();
    let ctx = FindContext {
        clnt: Box::new(S3ClientWrapper {
            client: s3_client,
            bucket: "xxxxx".to_string(),
        }),
        exec_cmd: Some("".to_string()),
        ignore_pattern: Some("".to_string()),
        name_pattern: opt.name.clone(),
        path_pattern: opt.path_match.clone(),
        regex_pattern: None,
        max_depth: 32,
        larger_size: large,
        smaller_size: small,
        print_fmt: Some("".to_string()),
        older_than: None,
        newer_than: None,
        watch: false,
        with_versions: false,
        match_tags: HashMap::new(),
        match_meta: HashMap::new(),
        target_alias: Some("minio".to_string()),
        target_url: Some(opt.path.clone()),
        target_full_url: None,
    };
    do_find(ctx.into()).await;
}

fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

struct FindContext {
    exec_cmd: Option<String>,
    ignore_pattern: Option<String>,
    name_pattern: Option<String>,
    path_pattern: Option<String>,
    regex_pattern: Option<Regex>,
    max_depth: u32,
    print_fmt: Option<String>,
    older_than: Option<String>,
    newer_than: Option<String>,
    larger_size: u64,
    smaller_size: u64,
    watch: bool,
    with_versions: bool,
    match_meta: HashMap<String, Option<Regex>>,
    match_tags: HashMap<String, Option<Regex>>,

    // Internal values
    target_alias: Option<String>,
    target_url: Option<String>,
    target_full_url: Option<String>,
    clnt: Box<dyn Client>, // Replace `Client` with the appropriate struct or interface in Rust
}

fn name_match(pattern: &str, path: &str) -> bool {
    // Attempt to match the pattern with the base of the path
    let path_base = Path::new(path)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("");
    let matched = match glob::Pattern::new(pattern) {
        Ok(glob_pattern) => glob_pattern.matches(path_base),
        Err(e) => {
            eprintln!("Unable to match with input pattern: {:?}", e);
            return false;
        }
    };

    if matched {
        return true;
    }

    // If no match, check each component in the path
    path.split('/').any(|component| component == pattern)
}

fn pattern_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    let pt = Pattern::new(&pattern).unwrap();
    return pt.matches(&text);
}

fn path_match(pattern: &str, path: &str) -> bool {
    Pattern::new(pattern).unwrap().matches(path)
}

fn trim_suffix_at_max_depth(
    start_prefix: &str,
    path: &str,
    separator: &str,
    max_depth: u32,
) -> String {
    if max_depth == 0 {
        return path.to_string();
    }

    // Remove the requested prefix from consideration.
    // max_depth is only considered for all other levels excluding the starting prefix.
    let path = path.trim_start_matches(start_prefix);
    let mut path_components: Vec<&str> = path.split(separator).collect();

    if path_components.len() >= max_depth as usize {
        path_components.truncate(max_depth as usize);
    }

    let mut result_components = vec![start_prefix];
    result_components.extend(path_components);

    result_components.join("")
}

fn get_aliased_path(ctx: &FindContext, path: &str) -> String {
    "".to_string()
    //     let separator = ctx.clnt.get_url().separator().to_string();
    //     let prefix_path = ctx.clnt.get_url().to_string();

    //     let mut aliased_path = String::new();

    //     if !ctx.target_alias.unwrap().is_empty() {
    //         // Use target alias and trim the prefix
    //         aliased_path = format!(
    //             "{}{}",
    //             ctx.target_alias.unwrap(),
    //             path.trim_start_matches(ctx.target_full_url.unwrap().trim_end_matches(&separator))
    //         );
    //     } else {
    //         aliased_path = path.to_string();

    //         // Look for the prefix path
    //         if let Some(i) = path.find(&prefix_path) {
    //             if i > 0 {
    //                 aliased_path = path[i..].to_string();
    //             }
    //         }
    //     }

    //     // Call trim_suffix_at_max_depth to apply the maxDepth trimming logic
    //     trim_suffix_at_max_depth(&ctx.target_url, &aliased_path, &separator, ctx.max_depth)
}

fn is_older(ti: SystemTime, older_ref: &str) -> bool {
    if older_ref.is_empty() {
        return false;
    }

    // Get the age of the object
    let object_age = SystemTime::now()
        .duration_since(ti)
        .unwrap_or(Duration::new(0, 0));

    // Parse the `older_ref` to a duration
    let older_than = parse_duration(older_ref);

    // Compare object age with older than reference
    object_age < older_than
}

fn parse_duration(older_ref: &str) -> Duration {
    // This is a basic example that assumes the format is something like "30s", "1m", etc.
    // You can enhance this to handle more complex cases as needed.
    if let Ok(seconds) = older_ref.parse::<u64>() {
        Duration::new(seconds, 0)
    } else {
        // Handle invalid input or fallback case
        Duration::new(0, 0)
    }
}

fn is_newer(ti: SystemTime, newer_ref: &str) -> bool {
    if newer_ref.is_empty() {
        return false;
    }

    // Calculate the age of the object
    let object_age = SystemTime::now()
        .duration_since(ti)
        .unwrap_or(Duration::new(0, 0));

    // Parse the 'newer_ref' string to a Duration
    let newer_than = parse_duration(newer_ref);

    // Compare the object's age with the duration specified in 'newer_ref'
    object_age >= newer_than
}

fn match_metadata_regex_maps(
    m: &HashMap<String, Option<Regex>>,
    v: &HashMap<String, String>,
) -> bool {
    for (k, reg_opt) in m {
        if let Some(reg) = reg_opt {
            // If regex is provided, check the value
            match v.get(k) {
                Some(val) => {
                    if !reg.is_match(val) {
                        return false;
                    }
                }
                None => {
                    // Try the "X-Amz-Meta-" prefixed key
                    let prefixed_key = format!("X-Amz-Meta-{}", k);
                    match v.get(&prefixed_key) {
                        Some(val) => {
                            if !reg.is_match(val) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
            }
        } else {
            // If regex is None, just check for an empty value
            if let Some(val) = v.get(k) {
                if !val.is_empty() {
                    return false;
                }
            }
        }
    }
    true
}

fn match_regex_maps(m: &HashMap<String, Option<Regex>>, v: &HashMap<String, String>) -> bool {
    for (k, reg_opt) in m {
        if let Some(reg) = reg_opt {
            // If regex is provided, check the value
            match v.get(k) {
                Some(val) => {
                    // Normalize the string and check if it matches the regex
                    let normalized_val = val.nfc().collect::<String>();
                    if !reg.is_match(&normalized_val) {
                        return false;
                    }
                }
                None => return false, // If key does not exist in the metadata map
            }
        } else {
            // If regex is None, just check for an empty value
            if let Some(val) = v.get(k) {
                if !val.is_empty() {
                    return false;
                }
            }
        }
    }
    true
}

fn match_find(ctx: &FindContext, file_content: &ContentMessage) -> bool {
    println!("key is 1: {}:", file_content.key.clone());
    let mut match_result = true;
    let mut prefix_path = ctx.target_url.clone();

    // Add separator only if targetURL doesn't already have separator
    //????????????
    //     if !prefix_path.starts_with(ctx.clnt.get_url_separator().to_string().as_str()) {
    //         prefix_path.push(ctx.clnt.get_url_separator());
    //     }

    // Trim the prefix such that we apply file path matching techniques
    let path = file_content.key.trim_start_matches(&prefix_path.unwrap());

    if match_result && ctx.ignore_pattern.is_some() {
        match_result &= !path_match(&ctx.ignore_pattern.as_ref().unwrap(), path);
        //println!("key is 2: {}:", file_content.key.clone());
    }

    if match_result && ctx.name_pattern.is_some() {
        match_result = name_match(&ctx.name_pattern.as_ref().unwrap(), path);
        //println!("name_pattern 1 {}:", match_result);
        //println!("key is 3: {}: {}", file_content.key.clone(), match_result);
    }

    if match_result && ctx.path_pattern.is_some() {
        match_result = path_match(&ctx.path_pattern.as_ref().unwrap(), path);
        //println!("key is 4: {}: {} path={}, path_pattern={}", file_content.key.clone(), match_result, path, ctx.path_pattern.as_ref().unwrap());
    }

    if match_result && ctx.regex_pattern.is_some() {
        match_result = ctx.regex_pattern.as_ref().unwrap().is_match(path);
        //println!("key is 5: {}:", file_content.key.clone());
    }

    if match_result && ctx.older_than.is_some() {
        match_result &= !is_older(
            file_content.time.clone().into(),
            ctx.older_than.as_ref().unwrap(),
        );
        //println!("key is 6: {}:", file_content.key.clone());
    }

    if match_result && ctx.newer_than.is_some() {
        match_result &= !is_newer(
            file_content.time.clone().into(),
            ctx.newer_than.as_ref().unwrap(),
        );
        //println!("key is 7: {}:", file_content.key.clone());
    }

    if match_result && ctx.larger_size > 0 {
        match_result &= file_content.size > ctx.larger_size as i64;
        //println!("key is 8: {}:", file_content.key.clone());
    }

    if match_result && ctx.smaller_size > 0 {
        match_result &= file_content.size < ctx.smaller_size as i64;
        //println!("key is 9: {}:", file_content.key.clone());
    }

    if match_result && !ctx.match_meta.is_empty() {
        match_result &=
            match_metadata_regex_maps(&ctx.match_meta, &file_content.metadata.as_ref().unwrap());
        //println!("key is 10: {}:", file_content.key.clone());
    }

    if match_result && !ctx.match_tags.is_empty() {
        match_result &= match_regex_maps(&ctx.match_tags, &file_content.tags.as_ref().unwrap());
        //println!("key is 11: {}:", file_content.key.clone());
    }
    println!("key is 13: {}:{}", file_content.key.clone(), match_result);

    match_result
}

async fn find(ctx_ctx: &str, ctx: &FindContext, file_content: ContentMessage) {
    // Match the incoming content, if not matched return
    if !match_find(ctx, &file_content) {
        return;
    }

    // If execCmd is specified, execute the command and return
    if let Some(exec_cmd) = &ctx.exec_cmd {
        //exec_find(ctx_ctx, exec_cmd, &file_content);
        return;
    }

    // If printFmt is specified, format the output string
    if let Some(print_fmt) = &ctx.print_fmt {
        let formatted_content = strings_replace(ctx, print_fmt, &file_content);
        println!("Formatted output: {}", formatted_content);
    }

    // Print the message with file content
    //print_msg(FindMessage { file_content });
    println!("{}", file_content);
}

fn strings_replace(ctx: &FindContext, args: &str, file_content: &ContentMessage) -> String {
    let mut str = args.to_string();

    // Replace all instances of {}
    str = str.replace("{}", &file_content.key);

    // Replace all instances of {""}
    str = str.replace(r#"{""}"#, &format!("{:?}", file_content.key));

    // Replace all instances of {base}
    let base = Path::new(&file_content.key)
        .file_name()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    str = str.replace("{base}", &base);

    // Replace all instances of {"base"}
    str = str.replace(r#"{"base"}"#, &format!("{:?}", base));

    // Replace all instances of {dir}
    let dir = Path::new(&file_content.key)
        .parent()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    str = str.replace("{dir}", &dir);

    // Replace all instances of {"dir"}
    str = str.replace(r#"{"dir"}"#, &format!("{:?}", dir));

    // Replace all instances of {size}
    let size_humanized = human_bytes(file_content.size as f64);
    str = str.replace("{size}", &size_humanized);

    // Replace all instances of {"size"}
    str = str.replace(r#"{"size"}"#, &format!("{:?}", size_humanized));

    // Replace all instances of {time}
    let time_formatted = file_content.time.format("%Y-%m-%d %H:%M:%S").to_string();
    str = str.replace("{time}", &time_formatted);

    // Replace all instances of {"time"}
    str = str.replace(r#"{"time"}"#, &format!("{:?}", time_formatted));

    // Replace all instances of {url} and {"url"}
    //let share_url = get_share_url(ctx, &file_content.key);
    //str = str.replace("{url}", &share_url);
    //str = str.replace(r#"{"url"}"#, &format!("{:?}", share_url));

    // Replace all instances of {version} and {"version"}
    
    //str = str.replace("{version}", &file_content.version_id.as_ref().unwrap());
   // str = str.replace(r#"{"version"}"#, &format!("{:?}", file_content.version_id));

    str
}

async fn do_find(ctx: Arc<FindContext>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize list options
    let list_options = LsOptions {
        versions: false,
        incomplete: false,
        path: ctx.path_pattern.clone(),
        rewind: None,
        with_older_versions: ctx.with_versions,
        with_delete_markers: false,
        recursive: true,
        show_dir: "DirFirst".to_string(),
        with_metadata: !ctx.match_meta.is_empty() || !ctx.match_tags.is_empty(),
    };

    // Create a stream of content items
    let mut content_stream = ctx.clnt.list(&list_options).await;

    // Iterate over content items
    while let Some(content) = content_stream.recv().await {
        // if let Some(err) = content.err {
        //     eprintln!("Error: {}", err.to_string());
        //     continue;
        // }

        // Skip Glacier storage class items
        if let Some(storage_class) = &content.storage_class {
            if storage_class == "s3StorageClassGlacier" {
                continue;
            }
        }

        // Construct file content
        let file_content = ContentMessage {
            key: content.key,
            version_id: content.version_id,
            time: content.time,
            size: content.size,
            metadata: content.metadata,
            tags: content.tags,
            etag: "".to_string(),
            status: "".to_string(),
            filetype: "".to_string(),
            url: Some("".to_string()),
            is_delete_marker: Some(true),
            version_ord: Some(0),
            version_index: Some(0),
            storage_class: Some("".to_string()),
            is_directory: content.is_directory,
        };

        // Match the content (mocked function)
        if !match_find(&ctx, &file_content) {
            continue;
        }
        println!("src key is: {}", file_content.key.clone());

        // Execute command or format the output
        // if let Some(exec_cmd) = &ctx.exec_cmd {
        //     //???????
        //     //exec_find(exec_cmd, &file_content).await;
        //     continue;
        // }

        if let Some(print_fmt) = &ctx.print_fmt {
            let formatted_key = strings_replace(&ctx, print_fmt, &file_content);
            println!("Formatted Key: {}", formatted_key);
        } else {
            println!("Key: {}", file_content.key);
        }
    }

    Ok(())
}
