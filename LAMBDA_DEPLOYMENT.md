# AWS Lambda Deployment Guide

This guide will help you deploy your Rust Axum API to AWS Lambda using AWS SAM (Serverless Application Model).

## Prerequisites

1. **AWS CLI configured** with appropriate credentials
2. **SAM CLI** installed ([Installation Guide](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html))
3. **cargo-lambda** for building Rust Lambda functions
4. **Docker** (required by SAM CLI for local testing)

## Installation Steps

### 1. Install cargo-lambda

```bash
pip install cargo-lambda
```

Or install via cargo:

```bash
cargo install cargo-lambda
```

### 2. Install SAM CLI

**macOS:**

```bash
brew install aws-sam-cli
```

**Linux/Windows:** Follow the [official installation guide](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/install-sam-cli.html)

## Building and Deployment

### 1. Build the Lambda Function

Run the provided build script:

```bash
./build-lambda.sh
```

This script will:

- Install cargo-lambda if not present
- Build the Lambda binary with optimizations
- Create the deployment package

### 2. Deploy to AWS

For first-time deployment:

```bash
sam deploy --guided
```

This will prompt you for:

- Stack name (e.g., `rust-api-lambda`)
- AWS Region
- Confirmation for resource creation

For subsequent deployments:

```bash
sam deploy
```

## Local Testing

### 1. Start Local API Gateway

```bash
sam local start-api --port 8080
```

### 2. Test the endpoints

The API will be available at `http://localhost:8080`

Test your endpoints:

```bash
curl http://localhost:8080/user
curl http://localhost:8080/channel
curl http://localhost:8080/mqtt
```

## Architecture

The Lambda deployment includes:

- **Lambda Function**: `RustApiFunction` - Your Rust API wrapped in Lambda runtime
- **API Gateway**: HTTP API that routes all requests to the Lambda function
- **IAM Role**: Automatically created execution role for the Lambda function

## Environment Variables

You can add environment variables in the `template.yaml` file:

```yaml
Environment:
  Variables:
    RUST_LOG: info
    DATABASE_URL: your-database-url
    # Add more variables as needed
```

## Monitoring and Logs

- **CloudWatch Logs**: Automatically created log group `/aws/lambda/rust-api-lambda-RustApiFunction-*`
- **CloudWatch Metrics**: Lambda execution metrics available in AWS Console
- **X-Ray Tracing**: Can be enabled for distributed tracing

## Performance Considerations

- **Cold Start**: First request may take longer due to Lambda cold start
- **Memory**: Default is 512MB, adjust in `template.yaml` if needed
- **Timeout**: Default is 30 seconds, adjust based on your API requirements

## Troubleshooting

### Common Issues

1. **Build Errors**: Ensure you have the latest version of `cargo-lambda`
2. **Permission Issues**: Verify AWS credentials and IAM permissions
3. **Import Errors**: Check that all dependencies are properly included

### Debugging

Enable debug logging:

```bash
export RUST_LOG=debug
sam local start-api --port 8080
```

### View Logs

```bash
sam logs -n RustApiFunction --stack-name your-stack-name --tail
```

## Cost Optimization

- Lambda pricing is based on requests and execution time
- Consider using Lambda Provisioned Concurrency for consistent performance
- Monitor costs through AWS Cost Explorer

## Cleanup

To remove all AWS resources:

```bash
sam delete --stack-name your-stack-name
```

## Additional Resources

- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [SAM CLI Documentation](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
