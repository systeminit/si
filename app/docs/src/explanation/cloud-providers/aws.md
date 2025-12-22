---
outline:
  level: [1, 2, 3, 4]
---

# AWS (Amazon Web Services) Support in System Initiative

System Initiative supports AWS resources through the
[AWS Cloud Control API](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/what-is-cloudcontrolapi.html),
covering all services supported by that interface. If a resource or capability
is missing from the API, users can
[contact us](https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative)
so we can address the gap and extend coverage as needed.

## Authentication Methods

System Initiative supports the following AWS authentication methods:

- [Static Credentials](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
- [AWS IAM Identity Center (SSO)](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html)
- [AWS IAM Assume Role Policy](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html)

## Connecting System Initiative to your AWS Account

In order to use AWS from within System Initiative, you need to use an
`AWS Credential` component. Creating that component will then prompt you for the
credential setup supported above.

### Static Credentials

If you have generated static access keys for a user then you can set
`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` directly in the `AWS Credential`
component.

### AWS SSO Credentials

If you have generated credentials via the command `aws sso login` then you can
set `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`
directly in the `AWS Credential` component.

### Assuming a Role

System Initiative has an AWS account dedicated to be able to assume a role in
your organization. It has a single user in the account with permissions to do
nothing except assume roles in other accounts. The ARN of the user is
`arn:aws:iam::058264381944:user/si-access-prod-manager`.

Every workspace has a unique token assigned to it at creation. It is accessible
from within your workspace via the gear icon in the upper right. Please treat
this as a Secret.

You can create a role using the token and the arn above, with a trust
relationship as below and attach any required permissions you want to give the
role:

```json
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
in the AWS Credential component. You don't need to set an access key or a secret
access key. You can ensure it works by putting an AWS Credential on the canvas,
selecting the Secret you created and it will validate your credentials.
