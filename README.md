# **How to Deploy Your Rig App on AWS Lambda: A Step-by-Step Guide**

**TL;DR**

## **Introduction**

Welcome to the series **Deploy Your Rig Application**!  
Now that you've built a rig app, you may now be wondering what’s the best way to deploy the it for your users to enjoy. If so, you’ve come to the right place!   
This series will walk you through different ways to deploy the application using various services. In this tutorial we will be covering serverless deployment on AWS lambda, but stay tuned for deployment via AWS Fargate, AWS Amplify, ShuttleRS, and more! Depending on your use case, you’ll be able to choose the most fitting deployment option. Let’s go\!

## **Prerequisites**

Before we begin building, ensure you have the following:

* A functioning rig application. We will be using this [rig app](https://github.com/garance-buricatu/rig-aws-lambda) in our examples.   
* An AWS account  
* An Open AI api key

## **Let's get started**

### AWS Lambda use cases
You may want to deploy your Rust application on AWS lambda if it’s a task that can execute in under 15 mins or if your app is a REST API backend.

### AWS 🤝 Rust

AWS Lambda supports Rust through the use of the [OS-only runtime Amazon Linux 2023](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html) in conjunction with the [Rust runtime client](https://github.com/awslabs/aws-lambda-rust-runtime), a rust crate. 

#### REST API backend
* Use the [`lambda-http`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http) crate (from the runtime client) to write your function’s entrypoint. 
* Then, route traffic to your lambda via AWS API services like [Api Gateway](https://aws.amazon.com/api-gateway/), [App Sync](https://aws.amazon.com/pm/appsync), [VPC lattice](https://aws.amazon.com/vpc/lattice/), etc ... 
* If your lambda handles multple endpoints of your API, the crate [axum](https://github.com/tokio-rs/axum) faciliates the routing within the lambda.

#### Event based task
* Ex: your lambda is triggered by S3 to process an object that was just added to your bucket.
* Use the [`lambda_runtime`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-runtime) crate with [`lambda_events`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events) (from the runtime client) to write your function’s entrypoint.
* Then, invoke your function either via [`lambda invoke` command](https://docs.aws.amazon.com/cli/latest/reference/lambda/invoke.html) or with integrated AWS triggers (ie. S3 UploadObject trigger). 

**Note**: for both cases, the crate `tokio` must also be added to your project as the lambda runtime client uses `tokio` to handle asynchronous calls.

### Our example: A basic OpenAI agent

The mini app `flight-search-assistant-lambda` is a Rust program that is executed via the  `lambda_runtime`. It invokes an OpenAI agent, designed by `rig`, to help users find flights between airports. It is an event-based task that I will execute with the `lambda invoke` command.

##### My flight-search-assistant function is written, let’s deploy it to the cloud\!

There are *many* ways to deploy Rust lambdas to AWS. Some out of the box options include the AWS CLI, the [cargo lambda](https://www.cargo-lambda.info/guide/getting-started.html) CLI, the AWS SAM CLI, the AWS CDK, and more. You can also decide to create a Dockerfile for your app and use that container image in your Lambda function instead. See some useful examples [here](https://docs.aws.amazon.com/lambda/latest/dg/rust-package.html).

I used the cargo lambda CLI to deploy the code in `flight-seach-assistant-lambda` from my local machine to an AWS lambda:

| // Added my AWS credentials to my terminal// Created an AWS Lambda function named ‘flight-search-assistant’ with architecture arm64.cargo lambda build \--release \--arm64cargo lambda deploy flight-search-assistant \--binary-name flight\_search\_assistant\_lambda |
| :---- |

##### 

##### Let’s talk about some AWS Lambda metrics when using Rust

This is the code configuration of the `flight-search-assistant` function in AWS. The function’s code package (bundled code and dependencies required for lambda to run) includes the single rust binary called `bootstrap`, which is 3.9 MB\!![][image1]

Below is a screenshot of the Cloudwatch logs of the function after running it a couple hundred times with different memory sizes. As you can see, the average memory usage tends to be around 29 MB.  
![][image2]  
What about cold starts?

#### AWS Fargate (ECS)

You may want to deploy your Rust application on AWS Fargate if it is a long running process, like an always-on web server for example.

##### How does it work?

We don’t need to use any specific runtime client here to deploy our code into a container. Just provide a Dockerfile for your Rust program and it’s good to go\!

##### Our Example: A Discord chatbot

The app `discord_rig_bot` is an always-on Rust program that listens for messages on specific Discord channels, and invokes an OpenAI agent, designed by `rig`, to answer questions about `rig`. The agent RAGs `rig` documentation, including examples, guides, and FAQs to provide extra context to the Open AI model. RAGing is done using an in memory vector store offered by `rig` out of the box.

##### My discord\_rig\_bot program is written, let’s deploy it to the cloud\!

* First step is to write the dockerfile:

| FROM public.ecr.aws/docker/library/rust:latest as build // Use a basic rust image from ECRRUN apt-get updateWORKDIR /discord-rig-botCOPY . /discord-rig-botRUN cargo build \--releaseFROM public.ecr.aws/amazonlinux/amazonlinux:2023-minimal as runtimeCOPY \--from=build /discord-rig-bot/target/release/discord\_rig\_bot .CMD \["./discord\_rig\_bot"\] |
| :---- |

* Next step is to upload the image to ECR

| // Created a private repository in ECR named “\<your account number\>[.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot](http://.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot)” // Added my AWS credentials to my terminal docker build \-t 123456789000[.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot](http://.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot) . aws ecr get-login-password \--region us-east-1 | docker login \- \-username AWS \--password-stdin 123456789000.dkr.ecr.us-east-1.amazonaws.com docker push 123456789000[.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot](http://.dkr.ecr.us-east-1.amazonaws.com/discord-rig-bot) |
| :---- |

1. AWS Amplify  
2. Shuttle RS