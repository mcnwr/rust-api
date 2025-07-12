use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Error};
use aws_types::region::Region;
use std::env;

#[derive(Clone)]
pub struct DynamoDbConfig {
    pub client: Client,
}

impl DynamoDbConfig {
    pub async fn new() -> Result<Self, Error> {
        // Get AWS credentials from environment variables
        let access_key = env::var("ACCESS_KEY").expect("ACCESS_KEY must be set");
        let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let region_str = env::var("REGION").unwrap_or_else(|_| "us-east-1".to_string());

        // Create AWS credentials
        let credentials = aws_sdk_dynamodb::config::Credentials::new(
            access_key, secret_key, None, // session token
            None, // expiration
            "env-vars",
        );

        // Set up region provider - fix the region provider issue
        let region = Region::new(region_str);
        let region_provider = RegionProviderChain::default_provider().or_else(region);

        // Build the AWS config
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        // Create DynamoDB client
        let client = Client::new(&config);

        Ok(DynamoDbConfig { client })
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

// Helper functions for common DynamoDB operations
impl DynamoDbConfig {
    pub async fn list_tables(&self) -> Result<Vec<String>, Error> {
        let resp = self.client.list_tables().send().await?;
        Ok(resp.table_names.unwrap_or_default())
    }

    pub async fn table_exists(&self, table_name: &str) -> Result<bool, Error> {
        let tables = self.list_tables().await?;
        Ok(tables.contains(&table_name.to_string()))
    }
}
