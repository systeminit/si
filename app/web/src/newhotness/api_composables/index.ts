import { computed, ComputedRef, inject, Ref, ref, unref, watch } from "vue";
import { AxiosInstance, AxiosResponse } from "axios";
import { Span, trace } from "@opentelemetry/api";
import { RouteLocationRaw } from "vue-router";
import {
  authApiInstance as auth,
  sdfApiInstance as sdf,
} from "@/store/apis.web";
import { changeSetExists, muspelheimStatuses } from "@/store/realtime/heimdall";
import router from "@/router";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import * as heimdall from "@/store/realtime/heimdall";
import { assertIsDefined, Context } from "../types";
import * as rainbow from "../logic_composables/rainbow_counter";
import { reset } from "../logic_composables/navigation_stack";

export * as componentTypes from "./component";
export * as funcRunTypes from "./func_run";

const tracer = trace.getTracer("si-vue");

export enum routes {
  ActionAdd = "ActionAdd",
  ActionCancel = "ActionCancel",
  ActionFuncRunId = "ActionFuncRunId",
  ActionHold = "ActionHold",
  ActionRetry = "ActionRetry",
  ActionQueuedDetails = "ActionQueuedDetails",
  ApplyChangeSet = "ApplyChangeSet",
  AuditLogs = "AuditLogs",
  AuditLogsForComponent = "AuditLogsForComponent",
  ChangeSetApprovalStatus = "ChangeSetApprovalStatus",
  ChangeSetApprove = "ChangeSetApprove",
  ChangeSetCancelApprovalRequest = "ChangeSetCancelApprovalRequest",
  ChangeSetInitializeAndApply = "ChangeSetInitializeAndApply",
  ChangeSetRename = "ChangeSetRename",
  ChangeSetReopen = "ChangeSetReopen",
  ChangeSetRequestApproval = "ChangeSetRequestApproval",
  ComponentDebug = "ComponentDebug",
  ComponentsOnHead = "ComponentsOnHead",
  CreateComponent = "CreateComponent",
  CreateSecret = "CreateSecret",
  CreateView = "CreateView",
  DeleteComponents = "DeleteComponents",
  DeleteDefaultSubscriptionSource = "DeleteDefaultSubscriptionSource",
  DeleteView = "DeleteView",
  DuplicateComponents = "DuplicateComponents",
  EnqueueAttributeValue = "EnqueueAttributeValue",
  FuncRun = "FuncRun",
  FuncRunByAv = "FuncRunByAv",
  FuncRunLogs = "FuncRunLogs",
  GetFuncRunsPaginated = "GetFuncRunsPaginated",
  GetPublicKey = "GetPublicKey",
  RefreshAction = "RefreshAction",
  RestoreComponents = "RestoreComponents",
  SetDefaultSubscriptionSource = "SetDefaultSubscriptionSource",
  MgmtFuncRun = "MgmtFuncRun",
  MgmtFuncGetJobState = "MgmtFuncGetJobState",
  MgmtFuncGetLatest = "MgmtFuncGetLatest",
  UpdateComponentAttributes = "UpdateComponentAttributes",
  UpdateComponentManage = "UpdateComponentManage",
  UpdateComponentName = "UpdateComponentName",
  UpdateView = "UpdateView",
  UpgradeComponents = "UpgradeComponents",
  ViewAddComponents = "ViewAddComponents",
  ViewEraseComponents = "ViewEraseComponents",
  CreateChangeSet = "CreateChangeSet",
  AbandonChangeSet = "AbandonChangeSet",
  Workspaces = "Workspaces",
  ChangeSets = "ChangeSets",
  WorkspaceListUsers = "WorkspaceListUsers",
  GenerateApiToken = "GenerateApiToken",
  CheckDismissedOnboarding = "CheckDismissedOnboarding",
  DismissOnboarding = "DismissOnboarding",
}

/**
 * Once we implement the action API calls in here
 * Those routes would also exist in here
 */
const CAN_MUTATE_ON_HEAD: readonly routes[] = [
  routes.ActionCancel,
  routes.ActionHold,
  routes.ActionRetry,
  routes.ChangeSetRename,
  routes.CreateChangeSet,
  routes.AbandonChangeSet,
  routes.EnqueueAttributeValue,
  routes.RefreshAction,
  routes.ChangeSetInitializeAndApply,
  routes.GenerateApiToken,
  routes.DismissOnboarding,
  routes.CheckDismissedOnboarding,
] as const;

const COMPRESSED_ROUTES: readonly routes[] = [
  routes.UpdateComponentAttributes,
] as const;

const _routes: Record<routes, string> = {
  AbandonChangeSet: "/abandon",
  ActionAdd: "/action/add",
  ActionCancel: "/action/<id>/cancel",
  ActionFuncRunId: "/action/<id>/func_run_id",
  ActionHold: "/action/<id>/put_on_hold",
  ActionRetry: "/action/<id>/retry",
  ActionQueuedDetails: "/action/<id>/queued_details",
  ApplyChangeSet: "/apply",
  AuditLogs: "/audit-logs",
  AuditLogsForComponent: "/audit-logs/<componentId>",
  ChangeSetApprovalStatus: "/approval_status",
  ChangeSetApprove: "/approve",
  ChangeSetCancelApprovalRequest: "/cancel_approval_request",
  ChangeSetRename: "/rename",
  ChangeSetReopen: "/reopen",
  ChangeSetRequestApproval: "/request_approval",
  ComponentDebug: "/components/<id>/debug",
  CreateComponent: "/views/<viewId>/component",
  CreateSecret: "/components/<id>/secret",
  CreateView: "/views",
  DeleteComponents: "/components/delete",
  DeleteDefaultSubscriptionSource: "/components/<id>/attributes/default_source",
  DeleteView: "/views/<viewId>",
  DuplicateComponents: "/views/<viewId>/duplicate_components",
  EnqueueAttributeValue: "/components/<id>/attributes/enqueue",
  FuncRun: "/funcs/runs/<id>",
  FuncRunByAv: "/funcs/runs/latest_av/<id>/logs",
  FuncRunLogs: "/funcs/runs/<id>/logs",
  GetFuncRunsPaginated: "/funcs/runs/paginated",
  GetPublicKey: "/components/<id>/secret/public_key",
  MgmtFuncGetJobState: "/management/state/<funcRunId>",
  MgmtFuncGetLatest: "/management/component/<componentId>/latest",
  MgmtFuncRun: "/management/prototype/<prototypeId>/<componentId>/<viewId>",
  RefreshAction: "/action/refresh/<componentId>",
  RestoreComponents: "/components/restore",
  SetDefaultSubscriptionSource: "/components/<id>/attributes/default_source",
  UpdateComponentAttributes: "/components/<id>/attributes",
  UpdateComponentManage: "/components/<id>/manage",
  UpdateComponentName: "/components/<id>/name",
  UpdateView: "/views/<viewId>",
  UpgradeComponents: "/components/upgrade",
  ViewAddComponents: "/views/<viewId>/add_components",
  ViewEraseComponents: "/views/<viewId>/erase_components",

  // URLs without the default `change-set/:id` section
  ChangeSets: "/change-sets",
  CreateChangeSet: "/change-sets/create_change_set",
  ChangeSetInitializeAndApply: "/change-sets/create_initialize_apply",
  ComponentsOnHead: "/change-sets/components_on_head",
  WorkspaceListUsers: "/users",
  // Auth Api Endpoints
  Workspaces: "/workspaces",
  GenerateApiToken: "/authTokens",
  CheckDismissedOnboarding: "users/<userPk>/firstTimeModal",
  DismissOnboarding: "users/<userPk>/dismissFirstTimeModal",
} as const;

const AUTH_API_ROUTES = [
  _routes.Workspaces,
  _routes.GenerateApiToken,
  _routes.CheckDismissedOnboarding,
  _routes.DismissOnboarding,
];

// the mechanics
type Obs = {
  requested: Ref<boolean>;
  success: Ref<boolean>;
  inFlight: Ref<boolean>;
  bifrosting: Ref<boolean>;
  isWatched: boolean;
  span?: Span;
  label?: string;
  changeSetIdExecutedAgainst?: string;
};

type LabeledObs = Obs & Required<Pick<Obs, "label">>;

const setLabel = (obs: Obs, label: string): LabeledObs => {
  return {
    ...obs,
    label,
  };
};

export type ApiContext = Pick<
  Context,
  "changeSetId" | "workspacePk" | "onHead" | "user"
>;
export const apiContextForChangeSet = (
  ctx: Context,
  changeSetId: ChangeSetId,
): ApiContext => {
  return {
    workspacePk: ctx.workspacePk,
    user: ctx.user,
    changeSetId: computed(() => changeSetId),
    onHead: computed(() => ctx.headChangeSetId.value === changeSetId),
  };
};

export type DoResponse<R, A> = {
  req: AxiosResponse<R>;
  newChangeSetId: string | undefined;
  errorMessage: string | undefined;
  endpointArgs: A;
};

export class APICall<Response, Args> {
  workspaceId: string;
  changeSetId: string;
  path: string;
  ctx: ApiContext;
  canMutateHead: boolean;
  mustCompress: boolean;
  description: string;
  obs: LabeledObs;
  lobbyRequired: boolean;
  endpointArgs: Args;

  constructor(
    ctx: ApiContext,
    path: string,
    canMutateHead: boolean,
    mustCompress: boolean,
    description: string,
    obs: LabeledObs,
    endpointArgs: Args,
  ) {
    this.ctx = ctx;
    const workspaceId = unref(ctx.workspacePk);
    const changeSetId = unref(ctx.changeSetId);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.path = path;
    this.canMutateHead = canMutateHead;
    this.mustCompress = mustCompress;
    this.description = description;
    this.obs = obs;
    this.lobbyRequired = false;
    this.endpointArgs = endpointArgs;
  }

  pathWithArgs(): string {
    let path = this.path.slice(); // slice() acts like a clone
    if (this.endpointArgs) {
      Object.entries(this.endpointArgs).forEach(([k, v]) => {
        // tsc gets a little confused in that `k` could be a symbol?
        path = path.replace(`<${k.toString()}>`, v as string);
      });
    }
    return path;
  }

  url(): string {
    if (
      [
        _routes.Workspaces,
        _routes.CheckDismissedOnboarding,
        _routes.DismissOnboarding,
      ].includes(this.path)
    ) {
      return this.pathWithArgs();
    }
    if ([_routes.GenerateApiToken].includes(this.path)) {
      return `workspaces/${this.workspaceId}${this.pathWithArgs()}`;
    }
    if (
      [
        _routes.ChangeSets,
        _routes.CreateChangeSet,
        _routes.ChangeSetInitializeAndApply,
        _routes.WorkspaceListUsers,
        _routes.ComponentsOnHead,
      ].includes(this.path)
    ) {
      return `v2/workspaces/${this.workspaceId}${this.pathWithArgs()}`;
    }

    const API_PREFIX = `v2/workspaces/${this.workspaceId}/change-sets/${this.changeSetId}`;
    return `${API_PREFIX}${this.pathWithArgs()}`;
  }

  api(): AxiosInstance {
    if (AUTH_API_ROUTES.includes(this.path)) {
      return auth;
    } else return sdf;
  }

  async do<D = Record<string, unknown>>(
    method: string,
    data: D,
    params?: URLSearchParams,
  ): Promise<DoResponse<Response, Args>> {
    const start = performance.now();
    this.obs.requested.value = true;
    this.obs.inFlight.value = true;
    this.obs.bifrosting.value = true;
    this.obs.span = tracer.startSpan("watchedApi");
    this.obs.span.setAttributes({
      workspaceId: this.ctx.workspacePk.value,
      "api.on_head": this.ctx.onHead.value,
      userPk: this.ctx.user?.pk,
      "http.url": this.path,
      "api.label": this.obs.label,
      "api.is_watched": false,
      "http.method": method,
      "http.params": params?.toString(),
      "http.body": JSON.stringify(data),
    });
    let newChangeSetId;
    if (!this.canMutateHead && this.ctx.onHead.value) {
      newChangeSetId = await this.makeChangeSet();
    }
    this.obs.span.setAttributes({
      changeSetId: newChangeSetId ?? this.ctx.changeSetId.value,
    });
    rainbow.add(this.changeSetId, this.obs.label);
    this.obs.changeSetIdExecutedAgainst = this.changeSetId;

    let formattedData: D | ArrayBuffer = data;
    const headers: Record<string, string> = {};
    if (this.mustCompress) {
      const textEncoder = new TextEncoder();
      const readableStream = new ReadableStream({
        start(controller) {
          controller.enqueue(textEncoder.encode(JSON.stringify(data)));
          controller.close();
        },
      });

      const compressedStream = readableStream.pipeThrough(
        new CompressionStream("gzip"),
      );
      formattedData = await new Response(compressedStream).arrayBuffer();

      headers["Content-Encoding"] = "gzip";
    }

    const req = await this.api()<Response>({
      method,
      headers,
      url: this.url(),
      params,
      data: formattedData,
      validateStatus: (_status) => true, // don't throw exception on 4/5xxx
    });
    const end = performance.now();
    this.obs.span.setAttributes({
      "http.status_code": req.status,
      // "watched" API "duration" will contain "how long it took for the data to update"
      // this will just be the http call time + latency, good to have both
      "http.duration": end - start,
    });
    this.obs.inFlight.value = false;
    if (ok(req)) this.obs.success.value = true;
    if (!this.obs.isWatched) {
      rainbow.remove(this.changeSetId, this.obs.label);
      if (this.obs.span) this.obs.span.end();
    }

    // We have two shapes of errors from sdf: data.error as a string and data.error.message as a string
    // This code extracts both of those as an errorMessage value for the caller.
    let errorMessage;
    const err =
      req.data instanceof Object && "error" in req.data
        ? req.data.error
        : undefined;
    if (typeof err === "string") {
      errorMessage = err;
    } else if (
      err instanceof Object &&
      "message" in err &&
      typeof err.message === "string"
    ) {
      errorMessage = err.message;
    }

    return {
      req,
      newChangeSetId,
      errorMessage,
      endpointArgs: this.endpointArgs,
    };
  }

  async delete<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    return this.do("DELETE", data, params);
  }

  async get(params?: URLSearchParams) {
    this.obs.requested.value = true;
    this.obs.inFlight.value = true;
    const req = await this.api()<Response>({
      method: "GET",
      url: this.url(),
      params,
    });
    if (ok(req)) this.obs.success.value = true;
    this.obs.inFlight.value = false;
    return req;
  }

  async makeChangeSet() {
    const req = await this.api()<{ id: string }>({
      method: "POST",
      url: `v2/workspaces/${this.workspaceId}/change-sets/create_change_set`,
      data: { name: this.description },
    });
    if (req.status === 200) {
      const newChangeSetId = req.data.id;
      // following API calls will use the new changeSetId
      this.changeSetId = newChangeSetId;
      return newChangeSetId;
    } else if (req.status === 202) {
      this.lobbyRequired = true;
      const newChangeSetId = req.data.id;
      this.changeSetId = newChangeSetId;
      return newChangeSetId;
    } else throw new Error("Unable to make change set");
  }

  async put<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    return this.do("PUT", data, params);
  }

  async post<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    return this.do("POST", data, params);
  }

  // very odd, i tried having a private `innerPostPut` to pass `method = "POST" | "PUT"`
  // just to avoid duplicating the body... but something about the typing was breaking
  // and it didn't make sense... can revisit later
}

export const ok = (req: AxiosResponse) => {
  switch (req.status) {
    case 200:
    case 201:
      return true;
    default:
      return false;
  }
};

type ApiRequestStatus = {
  isRequested: boolean;
  isPending: boolean;
  isFirstLoad: boolean;
  isError: boolean;
  isSuccess: boolean;
};

export type EndpointArgs = Record<string, string>;
export type UseApi<A = EndpointArgs> = {
  ok: (req: AxiosResponse) => boolean;
  endpoint: <Response>(key: routes, args?: A) => APICall<Response, A>;
  inFlight: Ref<boolean, boolean>;
  bifrosting: Ref<boolean, boolean>;
  requestStatuses: ComputedRef<ApiRequestStatus>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setWatchFn: (fn: () => any) => void;
  navigateToNewChangeSet: (
    to: RouteLocationRaw,
    newChangeSetId: string,
  ) => Promise<void>;
};

export const useApi = <SpecificArgs extends EndpointArgs = EndpointArgs>(
  ctx?: ApiContext,
): UseApi<SpecificArgs> => {
  if (!ctx) ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  const obs: Obs = {
    requested: ref(false),
    success: ref(false),
    inFlight: ref(false),
    bifrosting: ref(false),
    isWatched: false,
  };

  // You have to run endpoint BEFORE you call setWatchFn or it will break
  let labeledObs: LabeledObs;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any
  let apiCall: APICall<any, SpecificArgs>;
  const endpoint = <Response>(key: routes, args?: SpecificArgs) => {
    const path = _routes[key];
    const needsArgs = path.includes("<") && path.includes(">");
    if (!args && needsArgs) {
      throw new Error(`Endpoint ${key}, ${path} requires arguments`);
    }
    assertIsDefined(ctx);

    const canMutateHead = CAN_MUTATE_ON_HEAD.includes(key);
    const mustCompress = COMPRESSED_ROUTES.includes(key);
    const argList = args ? Object.entries(args).flatMap((m) => m) : [];
    const desc = `${
      key === routes.DeleteComponents ? "Remove Component" : key
    } ${argList.join(": ")} by ${
      ctx.user?.name
    } on ${new Date().toLocaleDateString()}`;
    labeledObs = setLabel(obs, `${key}.${argList.join(".")}`);
    const call = new APICall<Response, SpecificArgs>(
      ctx,
      path,
      canMutateHead,
      mustCompress,
      desc,
      labeledObs,
      args ?? ({} as SpecificArgs),
    );
    apiCall = call;
    return call;
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const setWatchFn = (fn: () => any) => {
    labeledObs.isWatched = true;
    if (labeledObs.span) labeledObs.span.setAttribute("api.is_watched", true);
    const timeout = setTimeout(() => {
      if (labeledObs.span) {
        labeledObs.span.setAttributes({
          timed_out: true,
        });
        labeledObs.span.end();
      }
    }, 60000);
    watch(
      fn,
      () => {
        assertIsDefined(ctx);
        clearTimeout(timeout);
        labeledObs.bifrosting.value = false;
        rainbow.remove(
          labeledObs.changeSetIdExecutedAgainst ?? ctx.changeSetId.value,
          labeledObs.label,
        );
        if (labeledObs.span) labeledObs.span.end();
      },
      { once: true },
    );
  };

  const INTERVAL = 50; // 50ms
  const MAX_WAIT_IN_SEC = 10;
  const MAX_RETRY = (MAX_WAIT_IN_SEC * 1000) / INTERVAL; // "how many attempts to reach N seconds?"
  const navigateToNewChangeSet = async (
    to: RouteLocationRaw,
    newChangeSetId: string,
  ) => {
    await new Promise<void>((resolve, reject) => {
      let retry = 0;
      const interval = setInterval(async () => {
        assertIsDefined(ctx);
        if (retry >= MAX_RETRY) {
          clearInterval(interval);
          reject();
        }
        const exists = await changeSetExists(
          ctx.workspacePk.value,
          newChangeSetId,
        );
        if (exists) {
          clearInterval(interval);
          muspelheimStatuses.value[newChangeSetId] = true;
          resolve();
        }
        retry += 1;
      }, INTERVAL);
    });
    assertIsDefined(ctx);
    heimdall.showInterest(ctx.workspacePk.value, newChangeSetId);
    await router.push(to);
    reset();
  };

  const requestStatuses = computed<ApiRequestStatus>(() => {
    return {
      isRequested: obs.requested.value,
      isPending: obs.inFlight.value,
      isFirstLoad: false,
      isError: !obs.success.value,
      isSuccess: obs.success.value,
    };
  });

  return {
    ok,
    endpoint,
    inFlight: obs.inFlight,
    bifrosting: obs.bifrosting,
    requestStatuses,
    setWatchFn,
    navigateToNewChangeSet,
  };
};
