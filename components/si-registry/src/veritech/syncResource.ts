import {
  ActionReply,
  ResourceHealth,
  ResourceStatus,
  SyncResourceReply,
} from "./intelligence";

interface FailSyncResourceReplyRequest {
  resource: {
    state?: any;
    health: ResourceHealth;
    status: ResourceStatus;
  };
}

export function failSyncResourceReply(
  request: FailSyncResourceReplyRequest,
  opts?: {
    infoMsg?: string;
    errorMsg?: string;
    errorOutput?: string;
    status?: ResourceStatus;
    health?: ResourceHealth;
  },
): SyncResourceReply {
  const data = request.resource.state?.data ? request.resource.state.data : {};
  const state = { data };
  if (opts?.infoMsg) {
    state.data.infoMsg = opts.infoMsg;
  }
  if (opts?.errorMsg) {
    state.data.infoMsg = opts.errorMsg;
  }
  if (opts?.errorOutput) {
    state.data.errorOutput = opts.errorOutput;
  }
  const health: ResourceHealth = opts?.health
    ? opts.health
    : request.resource.health;
  const status: ResourceStatus = opts?.status
    ? opts.status
    : request.resource.status;

  const reply = {
    resource: {
      state,
      health,
      status,
    },
  };
  return reply;
}

export function failActionReply(
  request: FailSyncResourceReplyRequest,
  opts?: {
    infoMsg?: string;
    errorMsg?: string;
    errorOutput?: string;
    status?: ResourceStatus;
    health?: ResourceHealth;
  },
): ActionReply {
  const resource = failSyncResourceReply(request, opts).resource;

  const reply = {
    resource,
    actions: [],
  };
  return reply;
}
