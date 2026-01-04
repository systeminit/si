---
outline:
  level: [1, 2, 3, 4]
---

# DigitalOcean Support in System Initiative

System Initiative supports DigitalOcean resources through their
[OpenAPI Specification](https://api-engineering.nyc3.cdn.digitaloceanspaces.com/spec-ci/DigitalOcean-public.v2.yaml).
This allows us to automatically generate and maintain accurate coverage of
available services and resources.

We support authentication through:

- [API Tokens](https://docs.digitalocean.com/reference/api/create-personal-access-token/)

If this does not cover a use case you need, please
[contact us](https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative)
or send an email to <help@systeminit.com> so we can extend support as needed.

## Connecting System Initiative to your DigitalOcean Account

To use DigitalOcean from within System Initiative, you need to use a
`DigitalOcean Credential` component. Creating that component will then prompt
you for an API Token that you can use to create or discover your existing
infrastructure.
