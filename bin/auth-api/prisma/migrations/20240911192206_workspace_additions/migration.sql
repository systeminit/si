-- AlterTable
ALTER TABLE "workspaces" ADD COLUMN     "description" TEXT,
ADD COLUMN     "is_favourite" BOOLEAN NOT NULL DEFAULT false;
