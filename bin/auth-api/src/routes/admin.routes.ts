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

// List all non-deleted and non-quarantined workspaces
router.get("/list-workspace-ids", async (ctx) => {
  extractAdminAuthUser(ctx);

  const workspaces = await prisma.workspace.findMany({
    where: { deletedAt: null, quarantinedAt: null },
  });

  const workspaceIds = workspaces.map((workspace) => workspace.id);

  ctx.body = { workspaces: workspaceIds };
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
  const report = await prisma.$queryRaw<
  {
    id: string;
    email: string;
    nickname: string;
    signup_at: string;
    max_rum: number;
  }[]
  >`
    SELECT u.id,
           u.email,
           u.nickname,
           u.signup_at,
           MAX(e.owner_rum)::int AS max_rum
      FROM rum_change_events e
      JOIN users u ON u.id = e.owner_id
     WHERE (e.next_owner_event_timestamp <= ${monthStart} OR e.next_owner_event_timestamp IS NULL)
       AND (${monthStart} + INTERVAL '1 month' MONTH) > e.event_timestamp
    GROUP BY u.id, u.email, u.nickname, u.signup_at
    ORDER BY max_rum DESC, u.signup_at DESC
  `;

  ctx.body = report.map((entry) => ({
    id: entry.id,
    email: entry.email,
    nickname: entry.nickname,
    signupAt: entry.signup_at,
    maxRum: entry.max_rum,
  }));
});

router.post("/rum-data/bulk-upsert", async (ctx) => {
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      data: z.array(
        z.object({
          eventTimestamp: z.string(),
          rumChange: z.number(),

          workspaceId: z.string(),
          workspaceRum: z.number(),
          nextWorkspaceEventTimestamp: z.string().nullable(),
          prevWorkspaceEventTimestamp: z.string().nullable(),

          ownerId: z.string(),
          ownerRum: z.number(),
          nextOwnerEventTimestamp: z.string().nullable(),
          prevOwnerEventTimestamp: z.string().nullable(),
        }),
      ),
    }),
  );

  // Upsert each record
  const results = await Promise.all(
    reqBody.data.map((entry) => {
      const workspaceId_eventTimestamp = {
        workspaceId: entry.workspaceId,
        eventTimestamp: new Date(entry.eventTimestamp),
      };
      const update = {
        rumChange: entry.rumChange,
        workspaceRum: entry.workspaceRum,
        nextWorkspaceEventTimestamp: entry.nextWorkspaceEventTimestamp
          ? new Date(entry.nextWorkspaceEventTimestamp)
          : null,
        prevWorkspaceEventTimestamp: entry.prevWorkspaceEventTimestamp
          ? new Date(entry.prevWorkspaceEventTimestamp)
          : null,
        ownerId: entry.ownerId,
        ownerRum: entry.ownerRum,
        nextOwnerEventTimestamp: entry.nextOwnerEventTimestamp
          ? new Date(entry.nextOwnerEventTimestamp)
          : null,
        prevOwnerEventTimestamp: entry.prevOwnerEventTimestamp
          ? new Date(entry.prevOwnerEventTimestamp)
          : null,
      };

      return prisma.rumChangeEvent.upsert({
        where: {
          workspaceId_eventTimestamp,
        },
        update,
        create: {
          ...workspaceId_eventTimestamp,
          ...update,
        },
      });
    }),
  );

  ctx.body = {
    success: true,
    recordsProcessed: results.length,
  };
});
