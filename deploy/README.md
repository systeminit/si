# Deploying SI

When using the `Makefile` within this directory (`./deploy/`), a file named `docker-compose.env.yml` needs to exist.
Make targets will automatically create an minimal version of this file if it does not exist, but you can customize it further.
Here is an example:

```yaml
version: "3"
services:
  otel:
    environment:
      - HONEYCOMB_TOKEN=<token>
      - HONEYCOMB_DATASET=<dataset>
```

By default, the file will be created with the following contents:

```yaml
version: "3"
```

_Note:_ the `docker-compose.env.yml` file within this directory has been added to `.gitignore`.
This is because enviroment overrides may include tokens used for local development.
