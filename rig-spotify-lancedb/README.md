# Deploying Rig on Lambda with LanceDB vector store
LanceDB is a vector databases built from the ground-up to be efficient on disk. It supports multiple storage backends, including local NVMe, EBS, EFS and other third-party APIs that connect to the cloud. We will walk though three different storage options for your Rig app that uses LanceDb when deployed on AWS lambda.

## Local - Ephemeral storage
https://docs.aws.amazon.com/lambda/latest/dg/configuration-ephemeral-storage.html

Lambda ephemeral storage is temporary and unique to each execution environment, it is not intended for durable storage. In other words, any LanceDB store created during the lambda execution will be wiped when the function terminates. If can be set between 512 MB and 10240 MB.

Some uses cases may include:
* dynamic data loading with one-off RAG operation.

(see 2024-10-29T15:16:11.877-04:00 logs)

Initialize the DB in the /tmp folder of the lambda and configure the ephemeral storage property.
```
let db = lancedb::connect("/tmp").execute().await?;
```

Performance - lowest latency
Cost - highest cost

## Local - EFS
https://aws.amazon.com/blogs/compute/using-amazon-efs-for-aws-lambda-in-your-serverless-applications/

Serverless, elastic, shared file system designed to be consumed by other AWS services. Data in EFS is persisted and can be shared across lambda invocations. Supports up to 25,000 concurrent connections.

Cold starts - when a lambda' function execution environment is prepared for the first time, the file system is mounted. When the execution environment is warm from previous invocations, the EFS mount is already available.

Performance
Cost

## S3
Performance - highest latency
Cost - lowest cost