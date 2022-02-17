// Just barfing up everything here to go fast!

use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use aws_core::resource::{Resource, List};

type S3Client = aws_sdk_s3::client::Client;

pub struct S3 {
    pub(super) client: S3Client
}

impl S3 {
    pub async fn create() -> Self {
        let config = aws_config::from_env().load().await;

        Self { client: S3Client::new(&config) }
    }
}

#[async_trait]
impl List<Bucket> for S3 {
    type Error = aws_sdk_s3::SdkError<aws_sdk_s3::error::ListBucketsError>;
    
    async fn list(&self) -> Result<Vec<Arc<Bucket>>, Self::Error> {
        // no caching, that can be done generically
        let resp = self.client.list_buckets().send().await?;

        Ok(resp.buckets.unwrap().into_iter().map(|b| Arc::new(Bucket(b))).collect())
    }
}

// TODO: make struct + impl to auto-box resources into Arc
// though we may want to have more info on the structs?
pub struct Bucket(aws_sdk_s3::model::Bucket);

impl Deref for Bucket {
    type Target = aws_sdk_s3::model::Bucket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resource for Bucket {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(self.name().unwrap())
            .iri("arn:aws:s3:::".to_owned() + self.name().unwrap())
            .resource_type("S3Bucket")
            .build()
    }
}


type IAMClient = aws_sdk_iam::client::Client;

pub struct IAM {
    pub(super) client: IAMClient
}

impl IAM {
    pub async fn create() -> Self {
        let config = aws_config::from_env().load().await;

        Self { client: IAMClient::new(&config) }
    }
}

#[async_trait]
impl List<Role> for IAM {
    type Error = aws_sdk_iam::SdkError<aws_sdk_iam::error::ListRolesError>;
    
    async fn list(&self) -> Result<Vec<Arc<Role>>, Self::Error> {
        // no caching, that can be done generically
        let resp = self.client.list_roles().send().await?;

        Ok(resp.roles.unwrap().into_iter().map(|x| Arc::new(Role(x))).collect())
    }
}

pub struct Role(aws_sdk_iam::model::Role);


impl Deref for Role {
    type Target = aws_sdk_iam::model::Role;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resource for Role {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(self.role_name().unwrap())
            .iri("".to_owned() + self.arn().unwrap()) // FIX ROLE
            .resource_type("IamRole")
            .build()
    }
}


pub struct User(aws_sdk_iam::model::User);

// this easily be a blanket impl
impl Deref for User {
    type Target = aws_sdk_iam::model::User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resource for User {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(self.user_name().unwrap())
            .iri("".to_owned() + self.arn().unwrap()) 
            .set_description(self.user_id().map(String::from))
            .resource_type("IamUser")
            .build()
    }
}

#[async_trait]
impl List<User> for IAM {
    type Error = aws_sdk_iam::SdkError<aws_sdk_iam::error::ListUsersError>;
    
    async fn list(&self) -> Result<Vec<Arc<User>>, Self::Error> {
        // no caching, that can be done generically
        let resp = self.client.list_users().send().await.unwrap();

        Ok(resp.users.unwrap().into_iter().map(|x| Arc::new(User(x))).collect())
    }
}

type ECRClient = aws_sdk_ecr::client::Client;

pub struct ECR {
    pub(super) client: ECRClient
}

impl ECR {
    pub async fn create() -> Self {
        let config = aws_config::from_env().load().await;

        Self { client: ECRClient::new(&config) }
    }
}

pub struct ECRRepository(aws_sdk_ecr::model::Repository);

impl Deref for ECRRepository {
    type Target = aws_sdk_ecr::model::Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl Resource for ECRRepository {
    fn summary(&self) -> toolkits::model::ResourceSummary {
        toolkits::model::ResourceSummary::builder()
            .name(self.repository_name().unwrap())
            .description("".to_owned() + self.repository_uri().unwrap())
            .iri("".to_owned() + self.repository_arn().unwrap()) 
            .resource_type("EcrRepository")
            .build()
    }
}

#[async_trait]
impl List<ECRRepository> for ECR {
    type Error = aws_sdk_ecr::SdkError<aws_sdk_ecr::error::DescribeRepositoriesError>;
    
    async fn list(&self) -> Result<Vec<Arc<ECRRepository>>, Self::Error> {
        // no caching, that can be done generically
        let resp = self.client.describe_repositories().send().await.unwrap();

        Ok(resp.repositories.unwrap().into_iter().map(|x| Arc::new(ECRRepository(x))).collect())
    }
}
