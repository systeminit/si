-- AlterTable
ALTER TABLE "WorkspaceMembers" ADD COLUMN     "invited_at" TIMESTAMP(3);

-- Backfill the existing rows in the database
UPDATE "WorkspaceMembers"
SET "invited_at"  = NOW() where "invited_at" IS NULL;
