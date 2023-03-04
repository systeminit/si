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
- auto-restart is not currently triggered on .env file changes (see [issue](https://github.com/nodejs/node/issues/45467))