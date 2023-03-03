# SI Auth API

### Prisma / DB

Use `pnpx prisma` to run prisma commands locally. For example
- `pnpx prisma migrate dev --name something-descriptive` - generates and runs new migration based on prisma schema
- `pnpx prisma migrate reset` - wipes db, re-runs all migrations
- `pnpx prisma db push` - push changes directly to db without any migrations (good for experimentation)