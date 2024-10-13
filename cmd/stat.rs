#[derive(clap::Args, Debug)]
pub struct StatOptions {
    #[arg(help = "alias/bucket   (rustfs/bucketxyx)")]
    pub path: String,
    #[arg(long, help = "stat on older version(s)")]
    pub rewind: Option<String>,

    #[arg(long, help = "stat all versions")]
    pub versions: bool,

    #[arg(
        long = "version-id",
        short = 'v',
        help = "stat a specific object version"
    )]
    pub version_id: Option<String>,

    #[arg(long = "vid", help = "alias for version-id")]
    pub vid: Option<String>,

    #[arg(short, long, help = "stat all objects recursively")]
    pub recursive: bool,

    #[arg(short, long, help = "show extended bucket(s) stat")]
    pub verbose: bool,

    #[arg(long, help = "disable all LIST operations for stat")]
    pub no_list: bool,

    #[arg(
        long = "enc-c",
        help = "encrypt/decrypt objects using client provided keys (multiple keys can be provided)"
    )]
    pub enc_c: Vec<String>,

    #[arg(
        long = "config-dir",
        short = 'C',
        help = "path to configuration folder",
        default_value = "/home/ldy/.mc"
    )]
    pub config_dir: String,

    #[arg(short, long, help = "disable progress bar display")]
    pub quiet: bool,

    #[arg(
        long = "disable-pager",
        alias = "dp",
        help = "disable mc internal pager and print to raw stdout"
    )]
    pub disable_pager: bool,

    #[arg(long, help = "disable color theme")]
    pub no_color: bool,

    #[arg(long, help = "enable JSON lines formatted output")]
    pub json: bool,

    #[arg(long, help = "enable debug output")]
    pub debug: bool,

    #[arg(
        long,
        help = "resolves HOST[:PORT] to an IP address (e.g., minio.local:9000=10.10.75.1)"
    )]
    pub resolve: Option<String>,

    #[arg(long, help = "disable SSL certificate verification")]
    pub insecure: bool,

    #[arg(
        long = "limit-upload",
        help = "limits uploads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)"
    )]
    pub limit_upload: Option<String>,

    #[arg(
        long = "limit-download",
        help = "limits downloads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)"
    )]
    pub limit_download: Option<String>,
}

pub async fn handle_stat_command(opt: &StatOptions) {
    let _ = stat(opt).await;
}

fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

async fn stat(opt: &StatOptions) {
    // if opt.path.is_empty() {
    //     println!("path is empty");
    //     return Err("Path is empty".into());
    // }
    // let (alias, key) = split_first_part(&opt.path);
    // let (bucket, _) = split_first_part(key);
    // if bucket == "" {
    //     println!("bucket is null");
    //     return Err("Bucket not provided".into());
    // }

    // let cli = crate::s3::client::get_s3client_from_alias(alias)?;

    // //s3_client.create_bucket().bucket(bucket);
    // match cli.delete_bucket().bucket(bucket).send().await {
    //     Ok(_) => {
    //         println!("Bucket '{}' delete successfully", bucket);
    //         Ok(())
    //     }
    //     Err(err) => {
    //         println!("{}", err);
    //         Err(Box::new(err))
    //     }
    // }
}
