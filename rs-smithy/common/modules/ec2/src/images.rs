use std::sync::Arc;

use aws_core::resource::{Resource, List};
use aws_sdk_ec2::{model::Image, Client, SdkError};
use async_trait::async_trait;
use toolkits::model::ResourceSummary;

use crate::model::Ec2;

pub struct Ec2Image {
    summary: Image,
}

impl Ec2Image {
    pub fn new(summary: Image) -> Self {
        Self { summary }
    }
}

impl Resource for Ec2Image {
    fn summary(&self) -> ResourceSummary {
        ResourceSummary::builder()
            .name(self.summary.image_id().unwrap().to_owned())
            .detail(self.summary.image_type().map_or("", |x| x.as_str()))
            .description(self.summary.description().unwrap_or_default())
            .iri("aws:ec2/image".to_owned() + "/" + self.summary.image_id().unwrap())
            .resource_type("EC2Image") // maybe do the ARN if we can find it?
            .build()
    }
}

#[async_trait]
impl List<Ec2Image> for Ec2 {
    type Error = SdkError<aws_sdk_ec2::error::DescribeImagesError>;
    
    async fn list(&self) -> Result<Vec<Arc<Ec2Image>>, Self::Error> {
        let resp = self.client.describe_images()
            .owners("self")
            .owners("amazon")
            .filters(aws_sdk_ec2::model::Filter::builder().name("root-device-type").values("ebs").build())
            .filters(aws_sdk_ec2::model::Filter::builder().name("state").values("available").build())
            .filters(aws_sdk_ec2::model::Filter::builder().name("is-public").values("true").build())
            .filters(aws_sdk_ec2::model::Filter::builder().name("architecture").values("arm64").build())
            .send().await?;
        
        Ok(resp.images.unwrap().into_iter().map(|image| {
            Arc::new(Ec2Image::new(image))
        }).collect())
    }
}