-- AlterTable
ALTER TABLE "users" ADD COLUMN     "quarantined_at" TIMESTAMP(3);

-- AlterTable
ALTER TABLE "workspaces" ADD COLUMN     "quarantined_at" TIMESTAMP(3);
