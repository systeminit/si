import { unref, inject, ref, Ref, watch } from "vue";
import { AxiosResponse } from "axios";
import { trace, Span } from "@opentelemetry/api";
import { RouteLocationRaw } from "vue-router";
import { sdfApiInstance as sdf } from "@/store/apis.web";
import { changeSetExists } from "@/store/realtime/heimdall";
import router from "@/router";
import { assertIsDefined, Context } from "../types";
import * as rainbow from "../logic_composables/rainbow_counter";
import { reset } from "../logic_composables/navigation_stack";

export * as componentTypes from "./component";
export * as funcRunTypes from "./func_run";

const tracer = trace.getTracer("bifrost");

export enum routes {
  GetFuncRunsPaginated = "GetFuncRunsPaginated",
  FuncRun = "FuncRun",
  FuncRunByAv = "FuncRunByAv",
  FuncRunLogs = "FuncRunLogs",
  UpdateComponentAttributes = "UpdateComponentAttributes",
  UpdateComponentName = "UpdateComponentName",
  CreateComponent = "CreateComponent",
  CreateView = "CreateView",
  CreateSecret = "CreateSecret",
  GetPublicKey = "GetPublicKey",
  ActionAdd = "ActionAdd",
  ActionCancel = "ActionCancel",
  ActionHold = "ActionHold",
  ActionRetry = "ActionRetry",
}

/**
 * Once we implement the action API calls in here
 * Those routes would also exist in here
 */
const CAN_MUTATE_ON_HEAD: readonly routes[] = [
  routes.ActionCancel,
  routes.ActionHold,
  routes.ActionRetry,
] as const;

const _routes: Record<routes, string> = {
  GetFuncRunsPaginated: "/funcs/runs/paginated",
  FuncRun: "/funcs/runs/<id>",
  FuncRunByAv: "/funcs/runs/latest_av/<id>/logs",
  FuncRunLogs: "/funcs/runs/<id>/logs",
  UpdateComponentAttributes: "/components/<id>/attributes",
  UpdateComponentName: "/components/<id>/name",
  CreateComponent: "/views/<viewId>/component",
  CreateView: "/views",
  CreateSecret: "/components/<id>/secret",
  GetPublicKey: "/components/<id>/secret/public_key",
  ActionAdd: "/action/add",
  ActionCancel: "/action/<id>/cancel",
  ActionHold: "/action/<id>/put_on_hold",
  ActionRetry: "/action/<id>/retry",
} as const;

// the mechanics
type Obs = {
  inFlight: Ref<boolean>;
  bifrosting: Ref<boolean>;
  isWatched: boolean;
  span?: Span;
  label?: string;
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
  description: string;
  obs: LabeledObs;
  lobbyRequired: boolean;

  constructor(
    ctx: Context,
    path: string,
    canMutateHead: boolean,
    description: string,
    obs: LabeledObs,
  ) {
    this.ctx = ctx;
    const workspaceId = unref(ctx.workspacePk);
    const changeSetId = unref(ctx.changeSetId);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.path = path;
    this.canMutateHead = canMutateHead;
    this.description = description;
    this.obs = obs;
    this.lobbyRequired = false;
  }

  url(): string {
    const API_PREFIX = `v2/workspaces/${this.workspaceId}/change-sets/${this.changeSetId}`;
    return `${API_PREFIX}${this.path}`;
  }

  async get(params?: URLSearchParams) {
    this.obs.inFlight.value = true;
    const req = await sdf<Response>({
      method: "GET",
      url: this.url(),
      params,
    });
    this.obs.inFlight.value = false;
    return req;
  }

  async makeChangeSet() {
    const req = await sdf<{ id: string }>({
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
    this.obs.inFlight.value = true;
    this.obs.bifrosting.value = true;
    rainbow.add(this.ctx.changeSetId.value, this.obs.label);
    if (this.obs.isWatched) this.obs.span = tracer.startSpan("watchedApi");
    let newChangeSetId;
    if (!this.canMutateHead && this.ctx.onHead.value) {
      newChangeSetId = await this.makeChangeSet();
    }

    const req = await sdf<Response>({
      method: "PUT",
      url: this.url(),
      params,
      data,
    });
    this.obs.inFlight.value = false;
    return { req, newChangeSetId };
  }

  async post<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    this.obs.inFlight.value = true;
    this.obs.bifrosting.value = true;
    rainbow.add(this.ctx.changeSetId.value, this.obs.label);
    if (this.obs.isWatched) this.obs.span = tracer.startSpan("watchedApi");
    let newChangeSetId;
    if (!this.canMutateHead && this.ctx.onHead.value) {
      newChangeSetId = await this.makeChangeSet();
    }

    const req = await sdf<Response>({
      method: "POST",
      url: this.url(),
      params,
      data,
    });
    this.obs.inFlight.value = false;
    return { req, newChangeSetId };
  }

  // very odd, i tried having a private `innerPostPut` to pass `method = "POST" | "PUT"`
  // just to avoid duplicating the body... but something about the typing was breaking
  // and it didn't make sense... can revisit later
}

export const useApi = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  const ok = (req: AxiosResponse) => {
    switch (req.status) {
      case 200:
      case 201:
        return true;
      default:
        return false;
    }
  };

  const obs: Obs = {
    inFlight: ref(false),
    bifrosting: ref(false),
    isWatched: false,
  };

  // You have to run endpoint BEFORE you call setWatchFn or it will break
  let labeledObs: LabeledObs;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let apiCall: APICall<any>;
  const endpoint = <Response>(key: routes, args?: Record<string, string>) => {
    let path = _routes[key];
    const needsArgs = path.includes("<") && path.includes(">");
    if (!args && needsArgs)
      throw new Error(`Endpoint ${key}, ${path} requires arguments`);

    if (args)
      Object.entries(args).forEach(([k, v]) => {
        path = path.replace(`<${k}>`, v);
      });
    const canMutateHead = CAN_MUTATE_ON_HEAD.includes(key);
    const argList = args ? [...Object.entries(args)].flatMap((m) => m) : [];
    const desc = `${key} ${argList.join(": ")} by ${
      ctx.user?.name
    } on ${new Date().toLocaleDateString()}`;
    labeledObs = setLabel(obs, `${key}.${argList.join(".")}`);
    const call = new APICall<Response>(
      ctx,
      path,
      canMutateHead,
      desc,
      labeledObs,
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
        labeledObs.bifrosting.value = false;
        rainbow.remove(ctx.changeSetId.value, labeledObs.label);
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
          resolve();
        }
        retry += 1;
      }, INTERVAL);
    });
    if (apiCall.lobbyRequired) {
      if (typeof to === "string") to = "new-hotness-lobby";
      else if ("name" in to) to.name = "new-hotness-lobby";
      else throw new Error("Thanks, router");
    }
    await router.push(to);
    reset();
  };

  return {
    ok,
    endpoint,
    inFlight: obs.inFlight,
    bifrosting: obs.bifrosting,
    setWatchFn,
    navigateToNewChangeSet,
  };
};
