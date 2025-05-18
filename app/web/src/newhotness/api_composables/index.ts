import { unref, inject, ref, Ref } from "vue";
import { AxiosResponse } from "axios";
import { sdfApiInstance as sdf } from "@/store/apis.web";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ActionId,
  ActionKind,
  ActionPrototypeId,
  ActionResultState,
} from "@/api/sdf/dal/action";
import { assertIsDefined, Context } from "../types";

export type FuncRunId = string;
export type FuncRunLogId = string;
export type ContentHash = string;

export type FuncRunState =
  | "Created"
  | "Dispatched"
  | "Running"
  | "Postprocessing"
  | "Failure"
  | "Success";

export type FuncKind =
  | "action"
  | "attribute"
  | "authentication"
  | "codeGeneration"
  | "intrinsic"
  | "management";

export enum FuncBackendKind {
  Array,
  Boolean,
  Diff,
  Identity,
  Integer,
  JsAction,
  JsAttribute,
  JsAuthentication,
  Json,
  JsSchemaVariantDefinition,
  JsValidation,
  Map,
  Object,
  String,
  Unset,
  Validation,
  Management,
}

export enum FuncBackendResponseType {
  Action,
  Array,
  Boolean,
  CodeGeneration,
  Identity,
  Integer,
  Json,
  Map,
  Object,
  Qualification,
  SchemaVariantDefinition,
  String,
  Unset,
  Validation,
  Void,
  Management,
}
export interface FuncRun {
  id: FuncRunId;
  state: FuncRunState;
  actor?: string;
  componentId?: ComponentId;
  attributeValueId?: string;
  componentName?: string;
  schemaName?: string;
  actionId?: ActionId;
  actionPrototypeId?: ActionPrototypeId;
  actionKind?: ActionKind;
  actionDisplayName?: string;
  actionOriginatingChangeSetId?: ChangeSetId;
  actionResultState?: ActionResultState;
  backendKind: FuncBackendKind;
  backendResponseType: FuncBackendResponseType;
  functionName: string;
  functionDisplayName?: string;
  functionKind: FuncKind;
  functionDescription?: string;
  functionLink?: string;
  functionArgsCasAddress: ContentHash;
  functionCodeCasAddress: ContentHash;
  resultValueCasAddress?: ContentHash;
  resultUnprocessedValueCasAddress?: ContentHash;
  createdAt: string;
  updatedAt: string;
  functionArgs: unknown;
  functionCodeBase64: string;
  resultValue: unknown;
  logs?: FuncRunLog;
}
export interface OutputLine {
  stream: string;
  execution_id: string;
  level: string;
  group?: string;
  message: string;
  timestamp: string;
}

export interface FuncRunLog {
  id: FuncRunLogId;
  createdAt: string;
  updatedAt: string;
  funcRunID: FuncRunId;
  logs: OutputLine[];
  finalized: boolean;
}

// move all the above types out of here for cleanliness
// leave the types below, this is the API definition!

// the route & interface definitions
// follow the pattern to make it easier on the humans!

export type GetFuncRunsPaginatedResponse = {
  funcRuns: FuncRun[];
  nextCursor: string | null;
};
export type FuncRunResponse = { funcRun: FuncRun };

export type FuncRunLogsResponse = { logs: FuncRunLog };

export type UpdateComponentAttributesArgs = Record<
  AttributeJsonPointer,
  SetAttributeTo
>;

export type ComponentIdType =
  | {
      schemaType: string;
      schemaVariantId: string;
    }
  | {
      schemaType: string;
      schemaId: string;
    };
export type CreateComponentPayload = ComponentIdType & {
  parentId: null;
  x: "0";
  y: "0";
  height: "0";
  width: "0";
};
export const createComponentPayload = (
  idType: ComponentIdType,
): CreateComponentPayload => {
  if (
    ("schemaId" in idType && !idType.schemaId) ||
    ("schemaVariantId" in idType && !idType.schemaVariantId)
  )
    throw new Error("schemaId or schemaVariantId required");
  return {
    ...idType,
    parentId: null,
    x: "0",
    y: "0",
    height: "0",
    width: "0",
  };
};

// Things you can set an attribute to
export type SetAttributeTo =
  // Set attribute to a static JS value (can be any JSON--object, array, string, number, boolean, null)
  | unknown
  // Set attribute to a subscription (another component's value feeds it)
  | {
      $source: "subscription";
      component: ComponentId | string;
      path: AttributeJsonPointer;
    }
  // Unset the value by not passing "value" field
  | { $source: "value"; value?: undefined }
  // Set attribute to a static JS value (use this to safely set object values that could have "$source" property in them)
  | { $source: "value"; value: unknown };

// JSON pointer to the attribute, relative to the component root (e.g. /domain/IpAddresses/0 or /si/name)
export type AttributeJsonPointer = string;

export type UpdateComponentNameArgs = {
  name: string;
};

export enum routes {
  GetFuncRunsPaginated = "GetFuncRunsPaginated",
  FuncRun = "FuncRun",
  FuncRunByAv = "FuncRunByAv",
  FuncRunLogs = "FuncRunLogs",
  UpdateComponentAttributes = "UpdateComponentAttributes",
  UpdateComponentName = "UpdateComponentName",
  CreateComponent = "CreateComponent",
}

/**
 * Once we implement the action API calls in here
 * Those routes would also exist in here
 */
const CAN_MUTATE_ON_HEAD: readonly routes[] = [] as const;

const _routes: Record<routes, string> = {
  GetFuncRunsPaginated: "/funcs/runs/paginated",
  FuncRun: "/funcs/runs/<id>",
  FuncRunByAv: "/funcs/runs/latest_av/<id>/logs",
  FuncRunLogs: "/funcs/runs/<id>/logs",
  UpdateComponentAttributes: "/components/<id>/attributes",
  UpdateComponentName: "/components/<id>/name",
  CreateComponent: "/views/<viewId>/component",
} as const;

// the mechanics
export class APICall<Response> {
  workspaceId: string;
  changeSetId: string;
  path: string;
  ctx: Context;
  canMutateHead: boolean;
  description: string;
  inFlight: Ref<boolean>;

  constructor(
    ctx: Context,
    path: string,
    canMutateHead: boolean,
    description: string,
    inFlight: Ref<boolean>,
  ) {
    this.ctx = ctx;
    const workspaceId = unref(ctx.workspacePk);
    const changeSetId = unref(ctx.changeSetId);
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.path = path;
    this.canMutateHead = canMutateHead;
    this.description = description;
    this.inFlight = inFlight;
  }

  url(): string {
    const API_PREFIX = `v2/workspaces/${this.workspaceId}/change-sets/${this.changeSetId}`;
    return `${API_PREFIX}${this.path}`;
  }

  async get(params?: URLSearchParams) {
    this.inFlight.value = true;
    const req = await sdf<Response>({
      method: "GET",
      url: this.url(),
      params,
    });
    this.inFlight.value = false;
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
    } else throw new Error("Unable to make change set");
  }

  async put<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    this.inFlight.value = true;
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
    this.inFlight.value = false;
    return { req, newChangeSetId };
  }

  async post<D = Record<string, unknown>>(data: D, params?: URLSearchParams) {
    this.inFlight.value = true;
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
    this.inFlight.value = false;
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

  const inFlight = ref(false);
  const endpoint = <Response>(key: routes, args?: Record<string, string>) => {
    let path = _routes[key];
    if (args)
      Object.entries(args).forEach(([k, v]) => {
        path = path.replace(`<${k}>`, v);
      });
    const canMutateHead = CAN_MUTATE_ON_HEAD.includes(key);
    const desc = `${key} ${
      args ? [...Object.entries(args)].flatMap((m) => m).join(": ") : ""
    } by ${ctx.user?.name} on ${new Date().toLocaleDateString()}`;
    return new APICall<Response>(ctx, path, canMutateHead, desc, inFlight);
  };

  return { ok, endpoint, inFlight };
};
