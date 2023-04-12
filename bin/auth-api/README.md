# SI Auth API

### Prisma / DB

Use `pnpx prisma` to run prisma commands locally. For example

- `pnpx prisma migrate dev --name something-descriptive` - generates and runs new migration based on prisma schema
- `pnpx prisma migrate reset` - wipes db, re-runs all migrations
- `pnpx prisma db push` - push changes directly to db without any migrations (good for experimentation)

### JWT Signing Key

- `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key`
- `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`

### Config / env vars

- Config is loaded from .env files
- Put local overrides in gitignored .env.local
- on deployed environments override using actual env vars
- auto-restart is not currently triggered on .env file changes (
  see [issue](https://github.com/nodejs/node/issues/45467))

## Running auth stack locally

By default, our system is set up to hit the production auth stack because we still want real auth when running local dev
instances.

While working on the auth stack, we still need to run it locally and configure things to point to our local auth stack:

- update auth-api env vars in `bin/auth-api/.env.local`
    - fill in `AUTH0_CLIENT_SECRET` (get from 1pass?)
    - (OPTIONAL) set auth-api redis url to a locally running redis instance (ex: `REDIS_URL=127.0.0.1:6379`) only if
      needing to test redis. Falls back to in-memory storage...
- update web app env vars (`app/web/.env.local`) to point to local auth stack
  ```
    VITE_AUTH_API_URL=http://localhost:9001
    VITE_AUTH_PORTAL_URL=http://localhost:9000
  ```
- run the backend but using the local auth stack by setting env var `LOCAL_AUTH_STACK=1` (
  ex: `LOCAL_AUTH_STACK=1 pnpm run dev:backend`)
- run the db migrations (`pnpm run db:reset`) locally after booting your local database
- run the auth api `pnmp run dev` in this directory or `pnpm dev:auth-api` at the root
- run the auth portal `pnmp run dev` in `app/auth-portal` or `pnpm dev:auth-portal` at the root
- (or run both by running `pnpm run dev:auth` at the root)

## Deploy the Auth API to Production

The auth-api runs on [AWS ECS](https://aws.amazon.com/ecs/) and the auth-api image is hosted
on [AWS ECR](https://aws.amazon.com/ecr/). The ECS service is set to have 2 instances of the auth-api running. Each
instance of the auth-api uses
a [task definition](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definitions.html) to pull the
latest docker image for the auth-api. The auth-api is deploying
using [GitHub Actions](https://github.com/features/actions). To trigger a deployment from main, we use
a [workflow dispatch](https://github.blog/changelog/2020-07-06-github-actions-manual-triggers-with-workflow_dispatch/)
event in GitHub actions.

This workflow dispatch will build and deploy the auth-api docker image to ECS. It will then deploy a new version of the
auth-api in a [blue/green deployment](https://martinfowler.com/bliki/BlueGreenDeployment.html). This will create a new
instance of the auth-api and then take the old instance offline. This ensures we keep our
system [HA](https://www.digitalocean.com/community/tutorials/what-is-high-availability).

The deployment mechanism can be found
in [GitHub](https://github.com/systeminit/si/actions/workflows/deploy-auth-api.yml). To trigger a deployment click on
the `Run Workflow` button and then choose the branch from which to deploy.

This will build and push the image and queue a deployment to ECS.

NOTE - db migrations are not yet automatic, and are being triggered manually before deploying a new version. (This is very infrequent so far...)

If new environment variables are needed to be passed to the auth-api, then a new task definition needs to be created in
AWS ECS. When that task definition is created, the task definition can be associated with the service and a deployment
can be created as normal.
