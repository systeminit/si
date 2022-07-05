# Production

The production environment is running on a single [EC2](https://aws.amazon.com/ec2/) instance, which you can reach if
you are on the [Tailscale](https://tailscale.com/) network.
You need to have the `si_key` from [1Password](https://1password.com/) in your keychain.

> You can add the private key with `ssh-add`.

## Interactive

You can log in interactively with:

```bash
ssh fedora@prod-1
```

The environment is running via [docker-compose](https://docs.docker.com/compose/) out of the `deploy` directory.

## Updates

Updates are applied to production automatically, as new containers are tagged `stable`.
If production is up and running, there should be nothing to do in the common case.

## Re-deploy manually

If you need to re-deploy the stack manually (i.e. a migration was changed), then you can execute
the following make target:

```bash
make down prod
```

## Fully deploy manually

If, for some reason, you need to deploy the system manually, you can.
Use the top level make target:

```bash
make deploy-prod
```

You must have `rsync` installed on your workstation, and the ssh-key loaded.

