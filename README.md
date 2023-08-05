# Mastodon Status

_finished, deploying_

A Rust Lambda function that is used to deliver downtime messages for my [website](https://sachiniyer.com) to Mastodon.

Uses [megalodon-rs](https://docs.rs/megalodon/latest/megalodon/) and [aws lambda rust runtime](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main) to take results generated from a selenium script to post changes to  [@sachiniyerstatus@botsin.space](https://botsin.space/@sachiniyerstatus). 

I use AWS infra because I don't want to be dependent on my cluster for this.
