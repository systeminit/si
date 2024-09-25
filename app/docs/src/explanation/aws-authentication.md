---
outline:
  level: [2, 3, 4]
---

# Authenticating to AWS from System Initiative

System Initiative provides a number of ways to authenticate to AWS.

## Static Credentials

If you have generated static access keys for a user then you can set
`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` directly in the `AWS Credential`
asset.

## AWS SSO Credentials

If you have generated credentials via the command `aws sso login` then you can
set `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`
directly in the `AWS Credential` asset.

## Assuming a Role

System Initiative has an AWS account dedicated to be able to assume a role in
your organization. It has a single user in the account with permissions to do
nothing except assume roles in other accounts. The ARN of the user is
`arn:aws:iam::058264381944:user/si-access-prod-manager`.

Every workspace has a unique token assigned to it at creation. It is accessible
from within your workspace via the gear icon in the upper right. Please treat
this as a secret.

You can create a role using the token and the arn above, with a trust
relationship as below and attach any required permissions you want to give the
role:

```
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
               "AWS":"arn:aws:iam::058264381944:user/si-access-prod-manager"
            },
            "Action": "sts:AssumeRole",
            "Condition": {
                "StringEquals": {
                    "sts:ExternalId": "YOUR WORKSPACE TOKEN"
                }
            }
        }
    ]
}
```

Take the arn of the role that you created and set that in the `AssumeRole` field
in the AWS Credential asset. You don’t need to set an access key or a secret
access key. You can ensure it works by putting an AWS Credential on the asset,
selecting the secret you created and it will validate your credentials.
