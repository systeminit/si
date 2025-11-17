---
outline:
  level: [1, 2, 3, 4]
---

# AWS Support in System Initiative

Updated November 17, 2025.

System Initiative supports AWS resources through the
[AWS Cloud Control API](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/what-is-cloudcontrolapi.html),
covering all services supported by that interface. If a resource or capability
is missing from the API, users can
[contact us](https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative)
so we can address the gap and extend coverage as needed.

We support the following AWS authentication methods:

- [Static Credentials](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
- [AWS IAM Identity Center (SSO)](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html)
- [AWS IAM Assume Role Policy](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html)

## Connecting System Initiative to your AWS Account

In order to use AWS from within System Initiative, you need to use an
`AWS Credential` component. Creating that component will then prompt you for the
credential setup supported above. When setting up an assumeRolePolicy, you can
follow [our guide](/explanation/aws-authentication) to get started.
