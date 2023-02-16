# Fix Flow

This document contains information related to developing and running the "fix flow".
The "fix flow" is the end-to-end process of finding `Components` and their corresponding
`Resources` that have fallen out of sync and running `Fixes` to bring them back in sync.

## Troubleshooting Created AWS EC2 Instances

If you are testing the "fix flow" with new [AWS EC2 Instances](https://aws.amazon.com/ec2/),
you may want to ensure that they booted up properly.

First, ensure your [`AWS CLI`](https://aws.amazon.com/cli/) is configured properly and is set to
the correct [`region`](https://aws.amazon.com/about-aws/global-infrastructure/regions_az/).

Now, you can get the console output via the following command:

```shell
aws ec2 get-console-output --instance-id <INSTANCE-ID> --output text
```
