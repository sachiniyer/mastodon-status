# Mastodon Status

A Rust Lambda function that is used to deliver downtime messages for my [website](https://sachiniyer.com) to Mastodon.

## Design

Uses [megalodon-rs](https://docs.rs/megalodon/latest/megalodon/) and [aws lambda rust runtime](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main) to take results generated from a selenium script to post changes to  [@sachiniyerstatus@botsin.space](https://botsin.space/@sachiniyerstatus). 

I use AWS infra because I don't want to be dependent on my cluster for this.

## Running

You should be able to run/deploy this as well. Make sure to clone recursively.

### Development

1. copy `env.sample` to `env` and modify it
2. install [Cargo Lambda](https://www.cargo-lambda.info/)
3. run `cargo lambda watch --env-file .env`

### Deployment

1. copy `env.sample` to `env` and modify it
2. install [Cargo Lambda](https://www.cargo-lambda.info/)
3. to deploy `cargo lambda deploy --iam-role <IAM ROLE> --env-file .env <LAMBDA NAME> --binary-name mastodon-status`
