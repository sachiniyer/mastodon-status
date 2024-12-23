# Mastodon Status

A Rust Lambda function that is used to deliver downtime messages for my [website](https://sachiniyer.com) to Mastodon.

## Design

Uses [megalodon-rs](https://docs.rs/megalodon/latest/megalodon/) and [aws lambda rust runtime](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main) to publish status to  ~[@sachiniyerstatus@botsin.space](https://botsin.space/@sachiniyerstatus)~ [@sachiniyerstatus@mas.to](https://mas.to/@sachiniyerstatus). 

I use AWS infra because I don't want to be dependent on my cluster for this. However, I still use my [status api](https://status.sachiniyer.com) ([source](https://github.com/sachiniyer/status)) to get status on all my services. If that goes down, I assume the whole cluster is down.

Specifically, I have a lambda function that is triggered by a scheduled cloudwatch rule to run every ~30~ 5 minutes.

## Running

You should be able to run/deploy this as well. Make sure to clone recursively.

### Development

1. copy `env.sample` to `.env` and modify it
2. install [Cargo Lambda](https://www.cargo-lambda.info/)
3. run `cargo lambda watch --env-file .env`

### Deployment

1. copy `env.sample` to `.env` and modify it
2. install [Cargo Lambda](https://www.cargo-lambda.info/)
3. to deploy `cargo lambda deploy --iam-role <IAM ROLE> --env-file .env <LAMBDA NAME> --binary-name mastodon-status`
