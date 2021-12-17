# Production

The production environment is running on a single ec2 instance, which you can reach if you are on the tailscale
network. You need to have the `si_key` from 1password in your keychain.

## Interactive

You can log in interactively with:

```
$ ssh fedora@prod-1
```

The environment is running via docker-compose out of the `deploy` directory.

## Updates

Updates are applied to production automatically, as new containers are tagged `stable`. If production is up and running, there should be nnothing to do in the common case.

## Deploy manually

If, for some reason, you need to deploy the system manually, you can. Use the top level make target:

```
$ make deploy-prod
```

You must have `rsync` installed on your workstation, and the ssh-key loaded.

