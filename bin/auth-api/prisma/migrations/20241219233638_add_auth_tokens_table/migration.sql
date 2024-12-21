-- CreateTable
CREATE TABLE "AuthToken" (
    "id" CHAR(26) NOT NULL,
    "name" TEXT,
    "userId" CHAR(26) NOT NULL,
    "workspaceId" CHAR(26) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expiresAt" TIMESTAMP(3),
    "claims" JSONB NOT NULL,
    "lastUsedAt" TIMESTAMP(3),
    "lastUsedIp" TEXT,

    CONSTRAINT "AuthToken_pkey" PRIMARY KEY ("id")
);

-- AddForeignKey
ALTER TABLE "AuthToken" ADD CONSTRAINT "AuthToken_userId_fkey" FOREIGN KEY ("userId") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "AuthToken" ADD CONSTRAINT "AuthToken_workspaceId_fkey" FOREIGN KEY ("workspaceId") REFERENCES "workspaces"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
