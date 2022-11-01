# Staging Host Deployment Documentation

The files in this folder allow you to deploy an EC2
instance that automatically deploy the latest versions
of SI's containers, resetting the env on every update.

Right now, it's only bringing up a coreos instance with  
SI's containers on startup, but no auto-update via watchtower.

It can be started by, while on the folder containing this file,
running:

```
butane staging-1.yaml --pretty --strict --files-dir ../../ > staging-1.ign
terraform apply  -auto-approve
```

The way it's working right now, butane copies the deployment
docker compose files and makefile onto the server,
and executes it. The idea would be to, in the future,
execute each server via its own systemd unit, and have
watchtower setup with a pre update
[lifecycle hook](https://containrrr.dev/watchtower/lifecycle-hooks/)
that wipes all the data whenever sdf or the dal get updated