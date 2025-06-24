use aws_sign_v4;
use url::Url;

const CONTENT_HASH: &str = "X-Amz-Content-Sha256";
const X_DATE: &str = "X-Amz-Date";
const EMPTY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

// #[tokio::main]
pub async fn get_request(
    ak: String,
    sk: String,
    url: String,
    region: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // let data = "";
    // // 创建一个Sha256对象
    // let mut hasher = Sha256::new();

    // // 写入数据到哈希器
    // hasher.update(data);

    // // 计算哈希值并获取结果
    // let result = hasher.finalize();

    // // 将哈希值打印为十六进制格式
    // println!("SHA-256 hash: {:x}", result);
    let datetime = chrono::Utc::now();
    //let url = "http://127.0.0.1:9000/minio/admin/v3/info?metrics=false";

    let purl = Url::parse(&url)?;
    println!("host is:{}", purl.host_str().unwrap());

    let mut headers = reqwest::header::HeaderMap::new();

    let host = if let Some(port) = purl.port() {
        format!("{}:{}", purl.host_str().unwrap(), port)
    } else {
        purl.host_str().unwrap().to_string()
    };

    headers.insert("host", host.parse().unwrap());
    headers.insert(
        X_DATE,
        datetime
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
            .parse()
            .unwrap(),
    );

    headers.insert(CONTENT_HASH, EMPTY_HASH.to_string().parse().unwrap());

    let s = aws_sign_v4::AwsSign::new(
        "GET", &url, &datetime, &headers, &region, &ak, &sk, "s3", "",
    );
    let signature = s.sign();
    println!("{:#?}", signature);
    headers.insert(reqwest::header::AUTHORIZATION, signature.parse().unwrap());

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .headers(headers.to_owned())
        .body("")
        .send()
        .await?;

    //println!("Status: {}", res.status());
    let body = res.text().await?;
    //println!("Body:\n\n{}", body);
    Ok(body)
}

#[tokio::main]
async fn main() {
    let res = get_request(
        "12345678".to_string(),
        "12345678".to_string(),
        "http://127.0.0.1:9000/minio/admin/v3/info?metrics=false".to_string(),
        "us-east-1".to_string(),
    )
    .await;
    println!("\n");
    match res {
        Ok(response) => {
            println!("{}", response)
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
