use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::{Client, Config};
use dotenv::dotenv;
use std::env;
use std::fs;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let endpoint_url = env::var("R2_ENDPOINT_URL").expect("R2_ENDPOINT_URL is not set");
    let access_key = env::var("R2_STORAGE_ACCESS_KEY").expect("R2_STORAGE_ACCESS_KEY is not set");
    let secret_key = env::var("R2_STORAGE_SECRET_KEY").expect("R2_STORAGE_SECRET_KEY is not set");
    let bucket = env::var("R2_BUCKET_NAME").expect("R2_BUCKET_NAME is not set");
    let folder = env::var("R2_FOLDER_NAME").unwrap_or_else(|_| "".to_string());

    // Konfigurasi kredensial dan region (region-nya bebas asal konsisten, karena R2 ignore ini)
    let credentials = Credentials::new(&access_key, &secret_key, None, None, "static");
    let region = Region::new("auto");

    let config = Config::builder()
        .behavior_version_latest()
        .region(region)
        .endpoint_url(endpoint_url)
        .credentials_provider(credentials)
        .build();

    let client = Client::from_conf(config);

    // Baca file
    let file_path = "american_breakfast_update.jpeg";
    let file_bytes = fs::read(file_path).expect("Failed to read file");
    let key = format!("{}/{}", folder.trim_end_matches('/'), file_path);

    // Upload
    let resp = client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(file_bytes))
        .content_type("image/jpeg") // ini yang penting
        .content_disposition("inline") // biar preview
        .cache_control("public, max-age=31536000") // optional
        .acl(ObjectCannedAcl::PublicRead)
        .send()
        .await
        .expect("Upload failed");

    println!("✅ Upload berhasil!");
    println!(
        "🔗 Public URL (manual): https://{}/{}/{}",
        env::var("R2_PUBLIC_ENDPOINT_URL").unwrap_or_default(),
        bucket,
        key
    );
    println!("📦 ETag: {:?}", resp.e_tag);
}
