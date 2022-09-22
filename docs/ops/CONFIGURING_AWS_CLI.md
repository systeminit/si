# Aws CLI

Presently, lang js uses the aws cli client on the host machine to interact with AWS.

Since all our servers are fedora machines, you can install the client by running:

```bash
sudo dnf install -y awscli
```

and then

```bash
aws configure
```

passing the following responses to each prompt:

- AWS Access Key ID: The `Access Key` field on
  this [1password entry](https://start.1password.com/open/i?a=6FRDDOEI5JBKHJJAMQIKAEFWD4&v=y5uwcpkwsqeppqg4cwkxnnpwdm&i=mw3mygbdcd66pgn4hgkroicssi&h=systeminitiativeinc.1password.com)
- AWS Secret Access Key:  The `Access Key` field on the
  same [1password entry](https://start.1password.com/open/i?a=6FRDDOEI5JBKHJJAMQIKAEFWD4&v=y5uwcpkwsqeppqg4cwkxnnpwdm&i=mw3mygbdcd66pgn4hgkroicssi&h=systeminitiativeinc.1password.com)
  as above
- Default region name: `us-east-2`
- Default output format: Leave empty


