# SI Auth API

## Running auth stack locally

By default, our system is set up to hit the production auth stack because we still want real auth when running local dev
instances.

While working on the auth stack, we still need to run it locally and configure things to point to our local auth stack:

1. update auth-api env vars in `bin/auth-api/.env.local`
    - if you don't have .env.local yet, copy the `auth api local dev .env.local` key into the `.env.local` file.
    - fill in `AUTH0_CLIENT_SECRET` and `AUTH0_M2M_CLIENT_SECRET` and `STRIPE_API_KEY` (get from 1pass)
    - (OPTIONAL) set auth-api redis url to a locally running redis instance (ex: `REDIS_URL=127.0.0.1:6379`) only if
      needing to test redis. Falls back to in-memory storage...
2. update web app env vars (`app/web/.env.local`) to point to local auth stack
  ```
    VITE_AUTH_API_URL=http://localhost:9001
    VITE_AUTH_PORTAL_URL=http://localhost:9000
  ```
3. Run the dev:stop command and then dev:up with the auth api environment variables:

   ```
   buck2 run dev:stop
   SI_AUTH_API_URL=http://localhost:9001 SI_CREATE_WORKSPACE_PERMISSIONS=open buck2 run dev
   ```
4. *Quickly* after step #3 boots the local database, run the db migrations:
   ```bash
   pnpm run db:reset
   ```
5. Tilt dashboard: Enable [`auth-api`](http://localhost:10350/r/auth-api/overview) and then [`auth-portal`](http://localhost:10350/r/auth-portal/overview).

### Prisma / DB

Use `pnpm exec prisma` to run prisma commands locally. For example

- `pnpm exec prisma migrate dev --name something-descriptive` - generates and runs new migration based on prisma schema
- `pnpm exec prisma migrate reset` - wipes db, re-runs all migrations
- `pnpm exec prisma db push` - push changes directly to db without any migrations (good for experimentation)

### JWT Signing Key

### ES256 

- `ssh-keygen -t ecdsa -b 256 -m PEM -f jwtES256.key`
- `openssl ec -in jwtES256.key -pubout -outform PEM -out jwtES256.key.pub`

### RS256 (deprecated)

- `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key`
- `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`

### Config / env vars

- Config is loaded from .env files
- Put local overrides in gitignored .env.local
- on deployed environments override using actual env vars
- auto-restart is not currently triggered on .env file changes (
  see [issue](https://github.com/nodejs/node/issues/45467))

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

NOTE - db migrations are not yet automatic, and are being triggered manually before deploying a new version. (This is
very infrequent so far...)

If new environment variables are needed to be passed to the auth-api, then a new task definition needs to be created in
AWS ECS. When that task definition is created, the task definition can be associated with the service and a deployment
can be created as normal.
