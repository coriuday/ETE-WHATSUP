use anyhow::Result;

use crate::AppState;

pub struct StorageService<'a> {
    state: &'a AppState,
}

impl<'a> StorageService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn client(&self) -> &aws_sdk_s3::Client {
        &self.state.s3
    }

    /// Upload raw bytes to S3/MinIO
    pub async fn upload_bytes(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String> {
        self.client()
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

        let presigned = self
            .client()
            .get_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(expires_secs))?)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Download an object's bytes by key
    pub async fn download_bytes(&self, key: &str) -> Result<bytes::Bytes> {
        let output = self
            .client()
            .get_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .send()
            .await?;

        let data = output.body.collect().await.map(|d| d.into_bytes())?;
        Ok(data)
    }

    /// Delete an object
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client()
            .delete_object()
            .bucket(&self.state.config.s3_bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}
