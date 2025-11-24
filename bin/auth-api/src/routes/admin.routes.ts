import _ from "lodash";
import { z } from "zod";
import { InstanceEnvType, PrismaClient } from "@prisma/client";
import { getUserByEmail, getUserById } from "../services/users.service";
import {
  createProductionWorkspaceForUser,
  createWorkspace,
  getUserWorkspaces,
  patchWorkspace,
  SAAS_WORKSPACE_URL,
} from "../services/workspaces.service";
import { validate } from "../lib/validation-helpers";
import { tracker } from "../lib/tracker";
import { extractWorkspaceIdParamWithoutAuthorizing } from "./workspace.routes";
import { extractAdminAuthUser, router } from ".";

const prisma = new PrismaClient();

export type WorkspaceLookup = {
  firstName?: string | null;
  lastName?: string | null;
  email?: string | null;
  displayName: string;
  instanceUrl: string | null;
};
router.get("/workspaces/admin-lookup/:workspaceId", async (ctx) => {
  // Just for authorization, result is discarded
  extractAdminAuthUser(ctx);

  const workspace = await extractWorkspaceIdParamWithoutAuthorizing(ctx);
  const user = await getUserById(workspace.creatorUserId);

  const workspaceDetails: WorkspaceLookup = {
    firstName: user?.firstName,
    lastName: user?.lastName,
    email: user?.email,
    displayName: workspace.displayName,
    instanceUrl: workspace.instanceUrl,
  };

  ctx.body = workspaceDetails;
});

router.post("/workspaces/setup-production-workspace", async (ctx) => {
  // Just for authorization, result is discarded
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userEmail: z.string(),
    }),
  );

  const user = await getUserByEmail(reqBody.userEmail);
  if (user) {
    const userWorkspaces = await getUserWorkspaces(user.id);
    const hasDefaultWorkspace = _.head(
      _.filter(
        userWorkspaces,
        (w) => w.isDefault && w.creatorUserId === user.id,
      ),
    );

    const workspaceDetails = await createWorkspace(
      user,
      InstanceEnvType.SI,
      SAAS_WORKSPACE_URL,
      `${user.nickname}'s Production Workspace`,
      hasDefaultWorkspace === null || hasDefaultWorkspace === undefined,
      "",
    );

    ctx.body = {
      newWorkspace: workspaceDetails,
    };
  }
});

router.post("/workspaces/setup-production-workspace-by-userid", async (ctx) => {
  // Just for authorization, result is discarded
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userId: z.string(),
    }),
  );

  const newWorkspace = await createProductionWorkspaceForUser(reqBody.userId);
  if (newWorkspace) {
    ctx.body = {
      newWorkspace,
    };
  }
});

router.get("/workspaces/:workspaceId/ownerWorkspaces", async (ctx) => {
  extractAdminAuthUser(ctx);

  const workspace = await extractWorkspaceIdParamWithoutAuthorizing(ctx);

  ctx.body = {
    workspaceId: workspace.id,
    workspaceOwnerId: workspace.creatorUserId,
    workspaces: await getUserWorkspaces(workspace.creatorUserId),
  };
});

router.patch("/workspaces/:workspaceId/quarantine", async (ctx) => {
  const authUser = extractAdminAuthUser(ctx);

  const workspace = await extractWorkspaceIdParamWithoutAuthorizing(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      isQuarantined: z.boolean(),
    }),
  );

  const quarantineDate = new Date();
  if (reqBody.isQuarantined) {
    tracker.trackEvent(authUser, "quarantine_workspace", {
      quarantinedBy: authUser.email,
      quarantinedAt: quarantineDate,
      workspaceId: workspace.id,
    });
  } else {
    tracker.trackEvent(authUser, "unquarantine_workspace", {
      unQuarantinedBy: authUser.email,
      unQuarantinedAt: quarantineDate,
      workspaceId: workspace.id,
    });
  }

  const quarantinedAt = reqBody.isQuarantined ? quarantineDate : null;

  await patchWorkspace(
    workspace.id,
    workspace.instanceUrl,
    workspace.displayName,
    quarantinedAt,
    workspace.description,
    workspace.isFavourite,
    workspace.isHidden,
    workspace.approvalsEnabled,
  );

  ctx.body = await getUserWorkspaces(authUser.id);
});

export type RumReportEntry = {
  ownerPk: string;
  ownerEmail: string;
  ownerName: string;
  totalRum: number;
};

router.get("/rum-report", async (ctx) => {
  extractAdminAuthUser(ctx);

  const reqQuery = validate(
    ctx.request.query,
    z.object({
      month: z.string().optional(),
    }),
  );

  // Parse the month or default to current month
  const targetDate = reqQuery.month ? new Date(reqQuery.month) : new Date();
  // Set to first day of month at midnight UTC
  const monthStart = new Date(
    Date.UTC(
      targetDate.getUTCFullYear(),
      targetDate.getUTCMonth(),
      1,
      0,
      0,
      0,
      0,
    ),
  );

  // Query with raw SQL for efficient join, group, and sum
  const report = await prisma.$queryRaw<RumReportEntry[]>`
    SELECT
      u.id as "ownerPk",
      u.email as "ownerEmail",
      COALESCE(u.nickname, u.email) as "ownerName",
      SUM(wrm.max_rum)::int as "totalRum"
    FROM workspace_rum_months wrm
    JOIN workspaces w ON wrm.workspace_id = w.id
    JOIN users u ON w.creator_user_id = u.id
    WHERE wrm.month = ${monthStart}
    GROUP BY u.id, u.email, u.nickname
    ORDER BY "totalRum" DESC
  `;

  ctx.body = report;
});

export type WorkspaceRumData = {
  workspaceId: string;
  month: string; // ISO date string for first of month
  maxRum: number;
  maxRumDataEnd: string; // ISO date string
  rumDate: string; // ISO date string
};

router.post("/rum-data/bulk-upsert", async (ctx) => {
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      data: z.array(
        z.object({
          workspaceId: z.string(),
          month: z.string(),
          maxRum: z.number(),
          maxRumDataEnd: z.string(),
          rumDate: z.string(),
        }),
      ),
    }),
  );

  // Upsert each record
  const results = await Promise.all(
    reqBody.data.map((entry) => {
      return prisma.workspaceRumMonth.upsert({
        where: {
          workspaceId_month: {
            workspaceId: entry.workspaceId,
            month: new Date(entry.month),
          },
        },
        update: {
          maxRum: entry.maxRum,
          maxRumDataEnd: new Date(entry.maxRumDataEnd),
          maxRumDate: new Date(entry.rumDate),
        },
        create: {
          workspaceId: entry.workspaceId,
          month: new Date(entry.month),
          maxRum: entry.maxRum,
          maxRumDataEnd: new Date(entry.maxRumDataEnd),
          maxRumDate: new Date(entry.rumDate),
        },
      });
    }),
  );

  ctx.body = {
    success: true,
    recordsProcessed: results.length,
  };
});
