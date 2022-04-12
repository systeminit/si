import { Config } from "@/config";
import { SdfWs } from "@/api/sdf/ws";
import { fromFetch } from "rxjs/fetch";
import { from, mergeMap, Observable } from "rxjs";
import _ from "lodash";
import { Workspace } from "@/api/sdf/dal/workspace";
import { SessionService } from "@/service/session";

export class FetchError extends Error {
  response: Response;

  constructor(message: string, response: Response) {
    super(message);
    this.name = "FetchError";
    this.response = response;
  }
}

export interface SdfError {
  statusCode: number;
  message: string;
  code: number;
}

export interface ApiResponseError {
  error: SdfError;
}

export type ApiResponse<T> = (T & { error?: never }) | ApiResponseError;

export class SDF {
  baseUrl: URL;
  wsBaseUrl: URL;
  currentToken?: string;

  ws?: SdfWs;

  constructor(config: Config) {
    this.baseUrl = config.sdfBaseUrl;
    this.wsBaseUrl = config.sdfBaseWsUrl;
    this.startUpdate();
  }

  startUpdate() {
    if (!this.ws && this.token) {
      this.setupUpdate();
    }
  }

  setupUpdate() {
    const url = new URL(this.wsBaseUrl.toString());
    url.searchParams.set("token", `Bearer ${this.token}`);
    this.ws = new SdfWs(url.toString());
  }

  set token(token: SDF["currentToken"]) {
    this.currentToken = token;
    if (token) {
      localStorage.setItem("si-sdf-token", token);
      this.setupUpdate();
    } else {
      localStorage.removeItem("si-sdf-token");
    }
  }

  get token(): SDF["currentToken"] {
    if (this.currentToken) {
      return this.currentToken;
    } else {
      const storedToken = localStorage.getItem("si-sdf-token");
      if (storedToken) {
        this.currentToken = storedToken;
      }
      return this.currentToken;
    }
  }

  standard_headers(): Headers {
    const headers = new Headers();
    headers.set("Content-Type", "application/json");
    if (this.token) {
      headers.set("Authorization", `Bearer ${this.token}`);
    }
    const storedWorkspace = sessionStorage.getItem("workspace") ?? null;
    if (storedWorkspace !== null) {
      const workspace: Workspace = JSON.parse(storedWorkspace);
      if (workspace) {
        headers.set("WorkspaceId", `${workspace.id}`);
      }
    }
    const storedApplication =
      sessionStorage.getItem("applicationNodeId") ?? null;
    if (storedApplication !== null) {
      const applicationNodeId: number = JSON.parse(storedApplication);
      if (applicationNodeId) {
        headers.set("ApplicationId", `${applicationNodeId}`);
      }
    }
    return headers;
  }

  requestUrl(pathString: string): URL {
    let basePath;
    if (this.baseUrl.pathname.endsWith("/")) {
      basePath = this.baseUrl.pathname.slice(
        0,
        this.baseUrl.pathname.length - 1,
      );
    } else {
      basePath = this.baseUrl.pathname;
    }
    let requestPath;
    if (pathString.startsWith("/")) {
      requestPath = pathString.slice(1);
    } else {
      requestPath = pathString;
    }
    return new URL(`${basePath}/${requestPath}`, this.baseUrl);
  }

  get<T>(
    pathString: string,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    queryParams?: Record<string, any>,
  ): Observable<ApiResponse<T>> {
    const headers = this.standard_headers();

    const url = this.requestUrl(pathString);
    if (queryParams) {
      Object.keys(queryParams).forEach((key) => {
        if (_.isBoolean(queryParams[key])) {
          if (queryParams[key]) {
            url.searchParams.set(key, "1");
          } else {
            url.searchParams.set(key, "0");
          }
        } else {
          url.searchParams.set(key, String(queryParams[key]));
        }
      });
    }
    const request = new Request(url.toString(), {
      method: "GET",
      headers,
    });
    return this.send_request(request);
  }

  post<T>(
    pathString: string,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    args: Record<string, any>,
  ): Observable<ApiResponse<T>> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "POST",
      headers,
      body: JSON.stringify(args),
    });
    return this.send_request(request);
  }

  patch<T>(
    pathString: string,
    args: Record<string, unknown>,
  ): Observable<ApiResponse<T>> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "PATCH",
      headers,
      body: JSON.stringify(args),
    });
    return this.send_request(request);
  }

  delete<T>(pathString: string): Observable<ApiResponse<T>> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "DELETE",
      mode: "cors",
      headers,
    });
    return this.send_request(request);
  }

  send_request<T>(request: Request): Observable<ApiResponse<T>> {
    return fromFetch(request).pipe(
      mergeMap((response) => {
        if (response.status === 401) {
          return from(SessionService.logout());
        } else {
          return from(response.json());
        }
      }),
    );
  }
}

export function isSdfError(obj: unknown): obj is ApiResponseError {
  return !!(obj as ApiResponseError).error;
}
