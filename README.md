# How to Deploy Your Rig App on AWS Lambda: A Step-by-Step Guide

**TL;DR**
- **Rig**: The Fullstack Agent Framework in Rust
- **Developer-Friendly**: Intuitive API design, comprehensive documentation, and scalability from simple chatbots to complex AI systems.
- **Key Features**: Unified API across LLM providers (OpenAI, Cohere); Streamlined embeddings and vector store support; High-level abstractions for complex AI workflows (e.g., RAG); Leverages Rust's performance and safety; Extensible for custom implementations
- **Contribute**: Build with Rig, provide feedback, win $100
- **Resources**: [GitHub](https://github.com/0xPlaygrounds/rig), [Examples](https://github.com/0xPlaygrounds/awesome-rig), [Docs](https://docs.rs/rig-core/latest/rig/).

## Table of Contents
- [How to Deploy Your Rig App on AWS Lambda: A Step-by-Step Guide](#how-to-deploy-your-rig-app-on-aws-lambda-a-step-by-step-guide)
    - [Table of Contents](#table-of-contents)
    - [Introduction](#introduction)
    - [Prerequisites](#prerequisites)
    - [AWS Lambda Quick Overview](#aws-lambda-quick-overview)
        - [AWS and Rust](#aws-rust)
            - [REST API backend](#rest-api-backend)
            - [Event based task](#event-based-task)
    - [Rig Entertainer Agent App](#rig-entertainer-agent-app)
        - [Now let's deploy it!](#now-lets-deploy-it)
        - [Metrics on the cloud](#metrics-on-the-cloud)
            - [Deployment package](#deployment-package)
            - [Memory, CPU, and runtime](#memory-cpu-and-runtime)
            - [Cold starts](#cold-starts)
    - [Langchain Entertainer Agent App](#langchain-entertainer-agent-app)
        - [Deployment package](#deployment-package-1)
        - [Memory, CPU, and runtime](#memory-cpu-and-runtime-1)
        - [Cold starts](#cold-starts-1)
    - [Community and Ecosystem](#community-and-ecosystem)
    - [The Road Ahead: Rig's Future](#the-road-ahead-rigs-future)
    - [Conclusion and Call to Action](#conclusion-and-call-to-action)


## Introduction

Welcome to the series **Deploy Your Rig Application**!
Apps built with Rig can vary in complexity across three core dimensions: LLM usage, knowledge bases for RAG, and the compute infrastructure where the application is deployed. In this series, we’ll explore how different combinations of these dimensions can be configured for production use.   

Today, we’ll start with a simple Rig agent that uses the [OpenAI model GPT-4-turbo](https://platform.openai.com/docs/models/gpt-4o), does not rely on a vector store (ie.: no RAGing), and will be deployed on AWS Lambda. 

This blog will provide a step-by-step deployment guide for the simple Rig app, showcase performance metrics of the Rig app running on AWS Lambda, and compare these metrics with those of a [LangChain]((https://www.langchain.com)) app on the same platform.

💡 If you're new to Rig and want to start from the beginning or are looking for additional tutorials, check out our [blog series](https://rig.rs/build-with-rig-guide.html).

Let’s dive in!

## Prerequisites

Before we begin building, ensure you have the following:

* A clone of the [`rig-entertainer-lambda`](https://github.com/garance-buricatu/rig-aws/tree/master/rig-entertainer-lambda) crate (or your own Rig application).   
* An AWS account  
* An Open AI api key


## AWS Lambda Quick Overview
You might deploy your Rust application on AWS lambda if it’s a task that can execute in under 15 mins or if your app is a REST API backend.

### AWS 🤝 Rust

AWS Lambda supports Rust through the use of the [OS-only runtime Amazon Linux 2023](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html) (a lambda runtime) in conjunction with the [Rust runtime client](https://github.com/awslabs/aws-lambda-rust-runtime), a rust crate. 

#### REST API backend
* Use the [`lambda-http`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http) crate (from the runtime client) to write your function’s entrypoint. 
* Then, route traffic to your lambda via AWS API services like [Api Gateway](https://aws.amazon.com/api-gateway/), [App Sync](https://aws.amazon.com/pm/appsync), [VPC lattice](https://aws.amazon.com/vpc/lattice/), etc ... 
* If your lambda handles multiple endpoints of your API, the crate [axum](https://github.com/tokio-rs/axum) facilitates the routing within the lambda.

#### Event based task (15 mins max.)
* Your lambda function is invoked by some event with the event passed as the payload. For example, configure your S3 bucket to trigger the lambda function when a new object is added to the bucket. The function will receive the new object in the payload and can further process it.
* Use the [`lambda_runtime`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-runtime) crate with [`lambda_events`](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events) (from the runtime client) to write your function’s entrypoint.
* Then, invoke your function either via [`lambda invoke` command](https://docs.aws.amazon.com/cli/latest/reference/lambda/invoke.html) or with integrated AWS triggers (ie. S3 UploadObject trigger). 

**Note**: for both cases, the crate [`tokio`](https://docs.rs/tokio/latest/tokio/) must also be added to your project as the lambda runtime client uses `tokio` to handle asynchronous calls.

## Rig Entertainer Agent App 🤡

The crate [`rig-entertainer-lambda`](https://github.com/garance-buricatu/rig-aws-lambda/tree/master/rig-entertainer-lambda) implements a simple Rust program that is executed via the `lambda_runtime`. It invokes a `Rig` agent using the OpenAI API, to entertain users with jokes. It is an event-based task that I will execute with the `lambda invoke` command.

The main takeaway here is that the app's `Cargo.toml` file must include the following dependencies:   
1. `rig-core` (our rig crate)
2. `lambda_runtime`
3. `tokio`

### Now let's deploy it!

There are *many* ways to deploy Rust lambdas to AWS. Some out of the box options include the AWS CLI, the [cargo lambda](https://www.cargo-lambda.info/guide/getting-started.html) CLI, the AWS SAM CLI, the AWS CDK, and more. You can also decide to create a Dockerfile for your app and use that container image in your Lambda function instead. See some useful examples [here](https://docs.aws.amazon.com/lambda/latest/dg/rust-package.html).

In this blog, we'll use the cargo lambda CLI option to deploy the code in `rig-entertainer-rust` from my local machine to an AWS lambda:

```bash
# Add your AWS credentials to my terminal
# Create an AWS Lambda function named ‘rig-entertainer’ with architecture x86_64.

function_name='rig-entertainer'

cd rig-entertainer-lambda
cargo lambda build --release # Can define different architectures here with --arm64 for example
cargo lambda deploy $function_name # Since the name of the crate is the same as the the lambda function name, no need to specify a binary file
``` 

### Metrics on the cloud ☁️

#### Deployment package
This is the code configuration of the `rig-entertainer` function in AWS. The function’s code package (bundled code and dependencies required for lambda to run) includes the single rust binary called `bootstrap`, which is 3.2 MB.

![Deployment Package Rust](rig-entertainer-lambda/assets/rig-deployment-package.png)

#### Memory, CPU, and runtime
The image below gives metrics on memory usage and execution time of the function. Each row represents a single execution of the function. In **yellow** is the **total memory used**, in **red** is the amount of **memory allocated**, and in **blue** is the **runtime**.
Although the lambda has many configuration options for the memory ranging from 128MB to 1024MB, we can see that the average memory used by our app is **26MB**.
![Rig Cloudwatch logs](rig-entertainer-lambda/assets/rig-cw-logs.png)

Let's get more information on the metrics above by spamming the function and calculating averages. I invoked `rig-entertainer` 50 times for each memory configuration of 128MB, 256MB, 512MB, 1024MB using the [power tuner tool](https://github.com/alexcasalboni/aws-lambda-power-tuning) and the result of those invocations are displayed in the chart below. 

The x-axis is the memory allocation, and the y-axis is the average runtime over the 50 executions of `rig-entertainer`. 

**Q.** We know that the function uses on average only 26MB per execution (which is less than the minimum memory allocation of 128MB) so why should we test higher memory configurations?   
**A.** [vCPUs are added to the lambda in proportion to memory](https://docs.aws.amazon.com/lambda/latest/operatorguide/computing-power.html) so adding memory could still affect the performance.

However, we can see that adding memory to the function (and therefore adding computational power) does not affect its performance at all. Since the [cost of a lambda execution](https://aws.amazon.com/lambda/pricing/) is calculated in GB-seconds, we get the most efficient lambda for the lowest price! 
![Power Tuner Rust](rig-entertainer-lambda/assets/rig-power-tuner.png)

#### Cold starts ❄️
[Cold starts](https://docs.aws.amazon.com/lambda/latest/operatorguide/execution-environments.html) occur when the lambda function's execution environment needs to be booted up from scratch. This includes setting up the actual compute that the lambda function is running on, and downloading the lambda function code and dependencies in that environment.    
Cold start latency doesn't affect all function executions because once the lambda environment has been setup, it will be reused by subsequent executions of the same lambda.   

In the lambda cloudwatch logs, if a function execution requires a cold start, we see the `Init Duration` metric at the end of the execution. 

For `rig-entertainer`, we can see that the average cold start time is **90.9ms**:
![Rig cold starts](rig-entertainer-lambda/assets/rig-coldstarts.png)
Note that the function was affected by cold starts 9 times out of the 245 times it was executed, so **0.036%** of the time.

## Langchain Entertainer Agent App 🐍
I replicated the OpenAI entertainer agent using the [langchain](https://python.langchain.com/) python library in this [mini python app](https://github.com/garance-buricatu/rig-aws-lambda/tree/master/langchain-entertainer-lambda) which I also deployed to AWS Lambda in a function called `langchain-entertainer`.

Let's compare the metrics outlined above.

#### Deployment package

This is the code configuration of the `langchain-entertainer` function in AWS. The function’s code package is a zip file including the lambda function code and all dependencies required for the lambda program to run.
![Deployment Package Python](langchain-entertainer-lambda/assets/deployment-package.png)

#### Memory, CPU, and runtime
There are varying memory configurations from 128MB, 256MB, 512MB, to 1024MB for the lambda shown in the table below. When 128MB of memory is allocated, on average about **112MB** of memory is used, and when more more than 128MB is allocated, about **130MB** of memory is used and the **runtime is lower**.
![alt text](langchain-entertainer-lambda/assets/cw-logs.png)

Let's get some more averages for these metrics: I invoked `langchain-entertainer` 50 times for each memory configuration of 128MB, 256MB, 512MB, 1024MB using the [power tuner tool](https://github.com/alexcasalboni/aws-lambda-power-tuning) and the result of those invocations were plotted in the graph below. 

We can see that by increasing the memory allocation (and therefore computation power) of `langchain-entertainer`, the function becomes more performant (lower runtime). However, note that since you pay per GB-seconds, a more performant function is more expensive. 
![alt text](langchain-entertainer-lambda/assets/power-tuner.png)

#### Cold starts ❄️
For `langchain-entertainer`, the average cold start time is: **1,898.52ms**, ie. 20x as much as the rig app coldstart.
![Langchain cold starts](langchain-entertainer-lambda/assets/coldstarts.png)
Note that the function was affected by cold starts 6 times out of the 202 times it was executed, so **0.029%** of the time.


## Resources

Rig is an emerging project in the open-source community, and we're continuously expanding its ecosystem with new integrations and tools. We believe in the power of community-driven development and welcome contributions from developers of all skill levels.

Stay connected and contribute to Rig's growth:

- 📚 [Documentation](https://docs.rs/rig-core/latest/rig/): Comprehensive guides and API references
- 💻 [GitHub Repository](https://github.com/0xPlaygrounds/rig): Contribute, report issues, or star the project
- 🌐 [Official Website](https://rig.rs/): Latest news, tutorials, and resources

Join our [community](https://discord.com/invite/playgrounds) channel to discuss ideas, seek help, and collaborate with other Rig developers.

## The Road Ahead: Rig's Future

As we continue to develop Rig, we're excited about the possibilities. Our roadmap includes:

1. **Expanding LLM Provider Support**: Adding integrations for more LLM providers to give developers even more choices.
2. **Enhanced Performance Optimizations**: Continuously improving Rig's performance to handle larger-scale applications.
3. **Advanced AI Workflow Templates**: Providing pre-built templates for common AI workflows to accelerate development further.
4. **Ecosystem Growth**: Developing additional tools and libraries that complement Rig's core functionality.

We're committed to making Rig the go-to library for LLM application development in Rust, and your feedback is crucial in shaping this journey.

## Conclusion and Call to Action

Rig is transforming LLM-powered application development in Rust by providing:
- A unified, intuitive API for multiple LLM providers
- High-level abstractions for complex AI workflows
- Type-safe development leveraging Rust's powerful features
- Extensibility and seamless ecosystem integration

We believe Rig has the potential to significantly enhance developers' building of AI applications, and we want you to be part of this journey.

**Your Feedback Matters!** We're offering a unique opportunity to shape the future of Rig:

1. Build an AI-powered application using Rig.
2. Share your experience and insights via this [feedback form](https://bit.ly/Rig-Review).
3. Get a chance to win $100 and have your project featured in our showcase!

Your insights will directly influence Rig's development, helping us create a tool that truly meets the needs of AI developers. 🦀✨

Thanks for reading,  
Garance   
Full-stack developer @ [Playgrounds Analytics](https://playgrounds.network/)