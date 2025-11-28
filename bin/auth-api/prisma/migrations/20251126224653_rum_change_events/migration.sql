/*
  Warnings:

  - You are about to drop the `workspace_rum_months` table. If the table is not empty, all the data it contains will be lost.

*/
-- DropForeignKey
ALTER TABLE "workspace_rum_months" DROP CONSTRAINT "workspace_rum_months_workspace_id_fkey";

-- DropTable
DROP TABLE "workspace_rum_months";

-- CreateTable
CREATE TABLE "rum_change_events" (
    "event_timestamp" TIMESTAMP(3) NOT NULL,
    "rum_change" INTEGER NOT NULL,
    "workspace_id" TEXT NOT NULL,
    "workspace_rum" INTEGER NOT NULL,
    "prev_workspace_event_timestamp" TIMESTAMP(3),
    "next_workspace_event_timestamp" TIMESTAMP(3),
    "owner_id" TEXT NOT NULL,
    "owner_rum" INTEGER NOT NULL,
    "prev_owner_event_timestamp" TIMESTAMP(3),
    "next_owner_event_timestamp" TIMESTAMP(3),

    CONSTRAINT "rum_change_events_pkey" PRIMARY KEY ("workspace_id","event_timestamp")
);

-- CreateIndex
CREATE INDEX "rum_change_events_owner_id_event_timestamp_idx" ON "rum_change_events"("owner_id", "event_timestamp");

-- AddForeignKey
ALTER TABLE "rum_change_events" ADD CONSTRAINT "rum_change_events_workspace_id_fkey" FOREIGN KEY ("workspace_id") REFERENCES "workspaces"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "rum_change_events" ADD CONSTRAINT "rum_change_events_owner_id_fkey" FOREIGN KEY ("owner_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
