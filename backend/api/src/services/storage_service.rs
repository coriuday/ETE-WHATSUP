use anyhow::Result;
use aws_sdk_s3::{config::Credentials, Client, Config};
use aws_config::Region;

use crate::AppState;

pub struct StorageService<'a> {
    state: &'a AppState,
}

impl<'a> StorageService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn build_client(&self) -> Client {
        let creds = Credentials::new(
            &self.state.config.s3_access_key,
            &self.state.config.s3_secret_key,
            None,
            None,
            "whatsup-static",
        );

        let config = Config::builder()
            .credentials_provider(creds)
            .region(Region::new(self.state.config.s3_region.clone()))
            .endpoint_url(&self.state.config.s3_endpoint)
            .force_path_style(self.state.config.s3_force_path_style)
            .build();

        Client::from_conf(config)
    }

    /// Upload raw bytes to S3/MinIO
    pub async fn upload_bytes(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String> {
        let client = self.build_client();

        client
            .put_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .body(data.to_vec().into())
            .content_type(content_type)
            .send()
            .await?;

        let url = format!(
            "{}/{}/{}",
            self.state.config.s3_endpoint,
            self.state.config.s3_bucket,
            key
        );

        Ok(url)
    }

    /// Get a presigned URL for downloading an object
    pub async fn get_presigned_url(&self, key: &str, expires_secs: u64) -> Result<String> {
        use aws_sdk_s3::presigning::PresigningConfig;
        use std::time::Duration;

        let client = self.build_client();
        let presigned = client
            .get_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(expires_secs))?)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Delete an object
    pub async fn delete(&self, key: &str) -> Result<()> {
        let client = self.build_client();
        client
            .delete_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}
