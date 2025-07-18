import { unref, inject, ref, Ref, watch, computed } from "vue";
import { AxiosInstance, AxiosResponse } from "axios";
import { trace, Span } from "@opentelemetry/api";
import { RouteLocationRaw } from "vue-router";
import {
  sdfApiInstance as sdf,
  authApiInstance as auth,
} from "@/store/apis.web";
import { changeSetExists, muspelheimStatuses } from "@/store/realtime/heimdall";
import router from "@/router";
import { assertIsDefined, Context } from "../types";
import * as rainbow from "../logic_composables/rainbow_counter";
import { reset } from "../logic_composables/navigation_stack";

export * as componentTypes from "./component";
export * as funcRunTypes from "./func_run";

const tracer = trace.getTracer("si-vue");

export enum routes {
  ActionAdd = "ActionAdd",
  ActionCancel = "ActionCancel",
  ActionHold = "ActionHold",
  ActionRetry = "ActionRetry",
  ActionFuncRunId = "ActionFuncRunId",
  ApplyChangeSet = "ApplyChangeSet",
  ChangeSetRename = "ChangeSetRename",
  CreateComponent = "CreateComponent",
  CreateSecret = "CreateSecret",
  CreateView = "CreateView",
  DeleteComponents = "DeleteComponents",
  DeleteView = "DeleteView",
  DuplicateComponents = "DuplicateComponents",
  EraseComponentsFromView = "EraseComponentsFromView",
  FuncRun = "FuncRun",
  FuncRunByAv = "FuncRunByAv",
  FuncRunLogs = "FuncRunLogs",
  GetFuncRunsPaginated = "GetFuncRunsPaginated",
  GetPublicKey = "GetPublicKey",
  RestoreComponents = "RestoreComponents",
  MgmtFuncRun = "MgmtFuncRun",
  MgmtFuncGetLatest = "MgmtFuncGetLatest",
  UpdateComponentAttributes = "UpdateComponentAttributes",
  UpdateComponentManage = "UpdateComponentManage",
  UpdateComponentName = "UpdateComponentName",
  UpdateView = "UpdateView",
  UpgradeComponents = "UpgradeComponents",
  CreateChangeSet = "CreateChangeSet",
  AbandonChangeSet = "AbandonChangeSet",
  Workspaces = "Workspaces",
  ChangeSets = "ChangeSets",
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
] as const;

const COMPRESSED_ROUTES: readonly routes[] = [
  routes.UpdateComponentAttributes,
] as const;

const _routes: Record<routes, string> = {
  ActionAdd: "/action/add",
  ActionCancel: "/action/<id>/cancel",
  ActionHold: "/action/<id>/put_on_hold",
  ActionRetry: "/action/<id>/retry",
  ActionFuncRunId: "/action/<id>/func_run_id",
  ApplyChangeSet: "/apply",
  ChangeSetRename: "/rename",
  CreateComponent: "/views/<viewId>/component",
  CreateSecret: "/components/<id>/secret",
  CreateView: "/views",
  DeleteComponents: "/components/delete",
  DeleteView: "/views/<viewId>",
  DuplicateComponents: "/views/<viewId>/duplicate_components",
  EraseComponentsFromView: "/views/<viewId>/erase_components",
  FuncRun: "/funcs/runs/<id>",
  FuncRunByAv: "/funcs/runs/latest_av/<id>/logs",
  FuncRunLogs: "/funcs/runs/<id>/logs",
  GetFuncRunsPaginated: "/funcs/runs/paginated",
  GetPublicKey: "/components/<id>/secret/public_key",
  RestoreComponents: "/components/restore",
  MgmtFuncRun: "/management/prototype/<prototypeId>/<componentId>/<viewId>",
  MgmtFuncGetLatest: "/management/component/<componentId>/latest",
  UpdateComponentAttributes: "/components/<id>/attributes",
  UpdateComponentManage: "/components/<id>/manage",
  UpdateComponentName: "/components/<id>/name",
  UpdateView: "/views/<viewId>",
  UpgradeComponents: "/components/upgrade",
  // THESE ARE SPECIAL CASED & NOT V2
  CreateChangeSet: "/change_set/create_change_set",
  AbandonChangeSet: "/change_set/abandon_change_set",
  Workspaces: "/workspaces", // not a v2 url
  ChangeSets: "CHANGESETS", // a short v2 url
} as const;

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

export class APICall<Response> {
  workspaceId: string;
  changeSetId: string;
  path: string;
  ctx: Context;
  canMutateHead: boolean;
  mustCompress: boolean;
  description: string;
  obs: LabeledObs;
  lobbyRequired: boolean;

  constructor(
    ctx: Context,
    path: string,
    canMutateHead: boolean,
    mustCompress: boolean,
    description: string,
    obs: LabeledObs,
    changesetId?: string,
  ) {
    this.ctx = ctx;
    const workspaceId = unref(ctx.workspacePk);
    const changeSetId = changesetId ?? unref(ctx.changeSetId);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.path = path;
    this.canMutateHead = canMutateHead;
    this.mustCompress = mustCompress;
    this.description = description;
    this.obs = obs;
    this.lobbyRequired = false;
  }

  url(): string {
    if (
      [
        _routes.Workspaces,
        _routes.AbandonChangeSet,
        _routes.CreateChangeSet,
      ].includes(this.path)
    ) {
      return this.path;
    }
    if ([_routes.ChangeSets].includes(this.path)) {
      return `v2/workspaces/${this.workspaceId}/change-sets`;
    }
    const API_PREFIX = `v2/workspaces/${this.workspaceId}/change-sets/${this.changeSetId}`;
    return `${API_PREFIX}${this.path}`;
  }

  api(): AxiosInstance {
    if ([_routes.Workspaces].includes(this.path)) return auth;
    else return sdf;
  }

  async do<D = Record<string, unknown>>(
    method: string,
    data: D,
    params?: URLSearchParams,
  ) {
    this.obs.requested.value = true;
    this.obs.inFlight.value = true;
    this.obs.bifrosting.value = true;
    if (this.obs.isWatched) this.obs.span = tracer.startSpan("watchedApi");
    let newChangeSetId;
    if (!this.canMutateHead && this.ctx.onHead.value) {
      newChangeSetId = await this.makeChangeSet();
    }
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
    this.obs.inFlight.value = false;
    if (ok(req)) this.obs.success.value = true;
    if (!this.obs.isWatched) rainbow.remove(this.changeSetId, this.obs.label);

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

    return { req, newChangeSetId, errorMessage };
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

const ok = (req: AxiosResponse) => {
  switch (req.status) {
    case 200:
    case 201:
      return true;
    default:
      return false;
  }
};

export const useApi = (ctx?: Context) => {
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
  let apiCall: APICall<any>;
  const endpoint = <Response>(key: routes, args?: Record<string, string>) => {
    let path = _routes[key];
    const needsArgs = path.includes("<") && path.includes(">");
    if (!args && needsArgs)
      throw new Error(`Endpoint ${key}, ${path} requires arguments`);
    assertIsDefined(ctx);

    // There are some endpoints that can operate on a changeset even if a user is not using it.
    // Sending changesetId as an arg will override the changesetId for this request.
    let changesetId;
    if (args)
      Object.entries(args).forEach(([k, v]) => {
        if (k === "changesetId") {
          changesetId = v;
        }
        path = path.replace(`<${k}>`, v);
      });
    const canMutateHead = CAN_MUTATE_ON_HEAD.includes(key);
    const mustCompress = COMPRESSED_ROUTES.includes(key);
    const argList = args ? Object.entries(args).flatMap((m) => m) : [];
    const desc = `${key} ${argList.join(": ")} by ${
      ctx.user?.name
    } on ${new Date().toLocaleDateString()}`;
    labeledObs = setLabel(obs, `${key}.${argList.join(".")}`);
    const call = new APICall<Response>(
      ctx,
      path,
      canMutateHead,
      mustCompress,
      desc,
      labeledObs,
      changesetId,
    );
    apiCall = call;
    return call;
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const setWatchFn = (fn: () => any) => {
    labeledObs.isWatched = true;
    watch(
      fn,
      () => {
        assertIsDefined(ctx);
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
    await router.push(to);
    reset();
  };

  type ApiRequestStatus = {
    isRequested: boolean;
    isPending: boolean;
    isFirstLoad: boolean;
    isError: boolean;
    isSuccess: boolean;
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
