resource "aws_iam_role" "mastodon-status" {
  name               = "mastodon-status"
  assume_role_policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
        "Action": "sts:AssumeRole",
        "Principal": {
            "Service": "lambda.amazonaws.com"
        },
        "Effect": "Allow",
        "Sid": ""
        }
    ]
}
EOF
}

resource "aws_lambda_function" "mastodon-status" {
  filename         = "mastodon-status.zip"
  function_name    = "mastodon-status"
  role             = aws_iam_role.mastodon-status.arn
  handler          = "mastodon-status.lambda_handler"
  source_code_hash = base64sha256(file("mastodon-status.zip"))
  runtime          = "arn:aws:lambda:us-east-1::runtime:904c897d14442788d50f990427bbbf4e8df27838f33ffdc86013c6c1389b2bd4"
  timeout          = "60"
  memory_size      = "128"
}

resource "aws_cloudwatch_event_rule" "mastodon-status" {
  name                = "mastodon-status"
  description         = "Trigger mastodon-status every 5 minutes"
  schedule_expression = "rate(30 minutes)"
}

resource "aws_cloudwatch_event_target" "mastodon-status" {
  rule      = aws_cloudwatch_event_rule.mastodon-status.name
  target_id = "mastodon-status"
  arn       = aws_lambda_function.mastodon-status.arn
}

resource "aws_lambda_permission" "mastodon-status" {
  statement_id  = "AllowExecutionFromCloudWatch"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.mastodon-status.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.mastodon-status.arn
}
