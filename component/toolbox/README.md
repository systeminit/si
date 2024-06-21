# Toolbox

This container provides a suite of tools that can be used to support
our production and non-production sites.

Run

```bash
$ ./awsi.sh info
```

for more details.

## Prereqs

Most scripts require you to have an active AWS profile. In order to set
one up, Run

```bash
$ aws configure sso
SSO session name (Recommended): session-name
SSO start URL [None]: https://systeminit.awsapps.com/start
SSO region [None]: us-east-1
...
CLI profile name: a-profile-name-I-will-definitely-remember
```

When interacting with the scripts, if they require or ask for an AWS Profile,
the `CLI profile name` mentioned above is what is being referenced.
