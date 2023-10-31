-- CreateEnum
CREATE TYPE "RoleType" AS ENUM ('OWNER', 'COLLABORATOR');

-- AlterTable
ALTER TABLE "users" ADD COLUMN     "signup_at" TIMESTAMP(3),
ALTER COLUMN "auth0_id" DROP NOT NULL,
ALTER COLUMN "auth0_details" DROP NOT NULL,
ALTER COLUMN "nickname" DROP NOT NULL;

-- Set All Users To Already Be Signedup
UPDATE "users"
SET "signup_at" = NOW() where "signup_at" IS NULL;

-- CreateTable
CREATE TABLE "WorkspaceMembers" (
    "id" CHAR(26) NOT NULL,
    "user_id" TEXT NOT NULL,
    "workspace_id" TEXT NOT NULL,
    "role_type" "RoleType" NOT NULL,

    CONSTRAINT "WorkspaceMembers_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "WorkspaceMembers_user_id_workspace_id_key" ON "WorkspaceMembers"("user_id", "workspace_id");

-- AddForeignKey
ALTER TABLE "WorkspaceMembers" ADD CONSTRAINT "WorkspaceMembers_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "WorkspaceMembers" ADD CONSTRAINT "WorkspaceMembers_workspace_id_fkey" FOREIGN KEY ("workspace_id") REFERENCES "workspaces"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
