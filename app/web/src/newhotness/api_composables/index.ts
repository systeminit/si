import { Ref, unref, inject } from "vue";
import { AxiosResponse } from "axios";
import { sdfApiInstance as sdf } from "@/store/apis.web";
import { assertIsDefined, WSCS } from "../types";

export class APICall {
  workspaceId: string;
  changeSetId: string;
  path: string;
  url: string;

  constructor(
    workspaceId: string | Ref<string>,
    changeSetId: string | Ref<string>,
    path: string,
  ) {
    workspaceId = unref(workspaceId);
    changeSetId = unref(changeSetId);
    const API_PREFIX = `v2/workspaces/${workspaceId}/change-sets/${changeSetId}`;
    this.workspaceId = workspaceId;
    this.changeSetId = changeSetId;
    this.path = path;
    this.url = `${API_PREFIX}${this.path}`;
  }

  async get<T>(params?: URLSearchParams) {
    const req = await sdf<T>({
      method: "GET",
      url: this.url,
      params,
    });
    return req;
  }

  // For example... can make PUT when you need
  async post<T>(data: Record<string, unknown>, params?: URLSearchParams) {
    const req = await sdf<T>({
      method: "POST",
      url: this.url,
      params,
      data,
    });
    return req;
  }
}

export const useApi = () => {
  const wscs = inject<WSCS>("WSCS");
  assertIsDefined(wscs);

  const ok = (req: AxiosResponse) => {
    switch (req.status) {
      case 200:
      case 201:
        return true;
      default:
        return false;
    }
  };

  // FUTURE: keep a list of all the URL paths and types in here
  // make them accessable (e.g. endpoint.getFuncRun(id))
  const endpoint = (path: string) => {
    return new APICall(wscs.workspacePk, wscs.changeSetId, path);
  };

  return { ok, endpoint };
};
