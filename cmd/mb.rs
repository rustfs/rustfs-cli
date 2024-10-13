#[derive(clap::Args, Debug)]
pub struct MbOptions {
    #[arg(help = "alias/bucket   (rustfs/bucketxyx)")]
    pub path: String,
    /// Specify bucket region; defaults to 'us-east-1'
    #[clap(long, default_value = "us-east-1")]
    pub region: Option<String>,

    /// Ignore if bucket/directory already exists
    #[clap(long = "ignore-existing", short = 'p')]
    pub ignore_existing: bool,

    /// Enable object lock
    #[clap(long = "with-lock", short = 'l')]
    pub with_lock: bool,

    /// Enable versioned bucket
    #[clap(long = "with-versioning")]
    pub with_versioning: bool,

    /// Path to configuration folder (default: "/home/ldy/.mc")
    #[clap(long = "config-dir", short = 'C', default_value = "/home/ldy/.mc")]
    pub config_dir: String,

    /// Disable progress bar display
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// Disable mc internal pager and print to raw stdout
    #[clap(long = "disable-pager", alias = "dp")]
    pub disable_pager: bool,

    /// Disable color theme
    #[clap(long = "no-color")]
    pub no_color: bool,

    /// Enable JSON lines formatted output
    #[clap(long = "json")]
    pub json: bool,

    /// Enable debug output
    #[clap(long = "debug")]
    pub debug: bool,

    /// Resolves HOST[:PORT] to an IP address. Example: minio.local:9000=10.10.75.1
    #[clap(long = "resolve")]
    pub resolve: Option<String>,

    /// Disable SSL certificate verification
    #[clap(long = "insecure")]
    pub insecure: bool,

    /// Limits uploads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)
    #[clap(long = "limit-upload")]
    pub limit_upload: Option<String>,

    /// Limits downloads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)
    #[clap(long = "limit-download")]
    pub limit_download: Option<String>,
}

pub async fn handle_mb_command(opt: &MbOptions) {
    let _ = mb(opt).await;
}

fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

pub async fn mb(opt: &MbOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opt.path.is_empty() {
        println!("path is empty");
        return Err("Path is empty".into());
    }
    let (alias, key) = split_first_part(&opt.path);
    let (bucket, _) = split_first_part(key);
    if bucket == "" {
        println!("bucket is null");
        return Err("Bucket not provided".into());
    }

    let cli = crate::s3::client::get_s3client_from_alias(alias)?;
    println!("Successfully created S3 client.");
    //s3_client.create_bucket().bucket(bucket);
    match cli.create_bucket().bucket(bucket).send().await {
        Ok(_) => {
            println!(
                "Bucket '{}' created successfully in region '{:?}'.",
                bucket, opt.region
            );
            return Ok(());
        }
        Err(err) => {
            println!("{}", err);
            Err(Box::new(err))
        }
    }
}
