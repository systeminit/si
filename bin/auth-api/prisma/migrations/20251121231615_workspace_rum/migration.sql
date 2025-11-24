-- CreateTable
CREATE TABLE "workspace_rum_months" (
    "workspace_id" TEXT NOT NULL,
    "month" TIMESTAMP(3) NOT NULL,
    "max_rum" INTEGER NOT NULL,
    "max_rum_data_end" TIMESTAMP(3) NOT NULL,
    "rum_date" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "workspace_rum_months_pkey" PRIMARY KEY ("workspace_id","month")
);

-- AddForeignKey
ALTER TABLE "workspace_rum_months" ADD CONSTRAINT "workspace_rum_months_workspace_id_fkey" FOREIGN KEY ("workspace_id") REFERENCES "workspaces"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
