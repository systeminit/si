import { AuthToken, PrismaClient } from "@prisma/client";
import { JwtPayload } from "jsonwebtoken";
import { WorkspaceId } from "./workspaces.service";
import { normalizeSdfAuthTokenPayload, SdfAuthTokenPayload } from "./auth.service";

const prisma = new PrismaClient();

export type AuthTokenId = string;

export async function getAuthTokens(workspaceId: WorkspaceId) {
  return await prisma.authToken.findMany({
    where: { workspaceId },
    orderBy: { id: "desc" },
  });
}

export async function getAuthToken(id: AuthTokenId) {
  return await prisma.authToken.findUnique({
    where: { id },
  });
}

export async function reportAuthTokenAccess(id: AuthTokenId, fromIp: string) {
  return await prisma.authToken.update({
    where: { id },
    data: { lastUsedAt: new Date(), lastUsedIp: fromIp },
  });
}

export async function registerAuthToken(
  name: string | undefined,
  token: SdfAuthTokenPayload & JwtPayload,
) {
  if (token.jti === undefined) throw new Error(`No token ID in token ${token}`);

  const { userId, workspaceId, claims } = normalizePayload(token);
  return await prisma.authToken.create({
    data: {
      id: token.jti,
      name,
      userId,
      workspaceId,
      claims,
      createdAt: token.iat ? new Date(token.iat * 1000) : undefined,
      expiresAt: token.exp ? new Date(token.exp * 1000) : undefined,
    },
  });

  function normalizePayload(token: SdfAuthTokenPayload) {
    switch (token.version) {
      case "2": {
        const { userId, workspaceId, role } = normalizeSdfAuthTokenPayload(token);
        return { userId, workspaceId, claims: { role } };
      }
      case undefined: {
        const { user_pk: userId, workspace_pk: workspaceId } = token;
        return { userId, workspaceId, claims: { role: "web" } };
      }
      default:
        return token satisfies never; // never = unreachable
    }
  }
}

export async function updateAuthToken(id: AuthTokenId, data: Pick<Partial<AuthToken>, "name" | "revokedAt">) {
  return await prisma.authToken.update({
    where: { id },
    data,
  });
}

export async function deleteAuthToken(id: AuthTokenId) {
  const { count } = await prisma.authToken.deleteMany({
    where: { id },
  });
  return count > 0;
}

export async function revokeAllWorkspaceTokens(workspaceId: WorkspaceId) {
  const revokedAt = new Date();

  const tokensToRevoke = await prisma.authToken.findMany({
    where: {
      workspaceId,
      revokedAt: null,
    },
  });

  const { count } = await prisma.authToken.updateMany({
    where: {
      workspaceId,
      revokedAt: null,
    },
    data: { revokedAt },
  });

  return {
    count,
    tokensToRevoke,
  };
}
