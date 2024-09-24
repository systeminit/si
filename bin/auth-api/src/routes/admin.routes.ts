import _ from "lodash";
import { z } from "zod";
import { InstanceEnvType } from "@prisma/client";
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
import { extractWorkspaceIdParam } from "./workspace.routes";
import { extractAdminAuthUser, router } from ".";

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

  const workspace = await extractWorkspaceIdParam(ctx);
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

router.patch("/workspaces/:workspaceId/quarantine", async (ctx) => {
  const authUser = extractAdminAuthUser(ctx);

  const workspace = await extractWorkspaceIdParam(ctx);

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
  );

  ctx.body = await getUserWorkspaces(authUser.id);
});
