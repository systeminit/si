-- Revoke all tokens for workspaces that have been deleted
-- This ensures that tokens for deleted workspaces can no longer be used
UPDATE "AuthToken"
SET "revokedAt" = CURRENT_TIMESTAMP
WHERE "workspaceId" IN (
    SELECT "id"
    FROM "workspaces"
    WHERE "deleted_at" IS NOT NULL
)
AND "revokedAt" IS NULL;
