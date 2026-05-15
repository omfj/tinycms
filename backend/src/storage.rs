use anyhow::Context;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

#[derive(Clone)]
pub struct StorageClient {
    client: aws_sdk_s3::Client,
    pub bucket: String,
    pub base_url: String,
}

impl From<crate::schema::StorageConfig> for StorageClient {
    fn from(cfg: crate::schema::StorageConfig) -> Self {
        let region = cfg
            .region
            .or_else(|| std::env::var("S3_REGION").ok())
            .unwrap_or_else(|| "us-east-1".into());

        let access_key_id = cfg
            .access_key_id
            .or_else(|| std::env::var("S3_ACCESS_KEY_ID").ok())
            .unwrap_or_default();

        let secret_access_key = cfg
            .secret_access_key
            .or_else(|| std::env::var("S3_SECRET_ACCESS_KEY").ok())
            .unwrap_or_default();

        let endpoint = cfg.endpoint.or_else(|| std::env::var("S3_ENDPOINT").ok());

        let credentials =
            Credentials::new(&access_key_id, &secret_access_key, None, None, "tinycms");

        let mut builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(region.clone()));

        if let Some(ep) = &endpoint {
            builder = builder.endpoint_url(ep).force_path_style(true);
        }

        let base_url =
            endpoint.unwrap_or_else(|| format!("https://s3.{region}.amazonaws.com/{}", cfg.bucket));

        Self {
            client: aws_sdk_s3::Client::from_conf(builder.build()),
            bucket: cfg.bucket,
            base_url,
        }
    }
}

impl StorageClient {
    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .send()
            .await
            .context("S3 upload failed")?;

        Ok(format!("{}/{}", self.base_url.trim_end_matches('/'), key))
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("S3 delete failed")?;

        Ok(())
    }
}
