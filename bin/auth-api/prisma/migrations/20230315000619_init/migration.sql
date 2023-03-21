-- CreateEnum
CREATE TYPE "InstanceEnvType" AS ENUM ('LOCAL', 'PRIVATE', 'SI');

-- CreateTable
CREATE TABLE "users" (
    "id" CHAR(26) NOT NULL,
    "auth0_id" TEXT NOT NULL,
    "auth0_details" JSONB NOT NULL,
    "nickname" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "email_verified" BOOLEAN NOT NULL DEFAULT false,
    "first_name" TEXT,
    "last_name" TEXT,
    "picture_url" TEXT,
    "discord_username" TEXT,
    "github_username" TEXT,

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "workspaces" (
    "id" CHAR(26) NOT NULL,
    "instance_env_type" "InstanceEnvType" NOT NULL,
    "instance_url" TEXT,
    "display_name" TEXT NOT NULL,
    "creator_user_id" TEXT NOT NULL,

    CONSTRAINT "workspaces_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "tos_agreements" (
    "id" CHAR(26) NOT NULL,
    "user_id" TEXT NOT NULL,
    "tos_version_id" TEXT NOT NULL,
    "timestamp" TIMESTAMP(3) NOT NULL,
    "ip_address" TEXT NOT NULL,

    CONSTRAINT "tos_agreements_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "users_auth0_id_key" ON "users"("auth0_id");

-- CreateIndex
CREATE INDEX "users_email_idx" ON "users"("email");

-- CreateIndex
CREATE INDEX "workspaces_creator_user_id_idx" ON "workspaces"("creator_user_id");

-- CreateIndex
CREATE INDEX "tos_agreements_user_id_idx" ON "tos_agreements"("user_id");

-- AddForeignKey
ALTER TABLE "workspaces" ADD CONSTRAINT "workspaces_creator_user_id_fkey" FOREIGN KEY ("creator_user_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "tos_agreements" ADD CONSTRAINT "tos_agreements_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
