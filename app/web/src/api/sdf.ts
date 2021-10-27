// import _ from "lodash";
import { Config } from "@/config";
import { urlSafeBase64Encode } from "@/api/sdf/base64";
import { IListRequest, IListReply } from "@/api/sdf/model";
import { Update } from "@/api/sdf/model/update";

export class FetchError extends Error {
  response: Response;

  constructor(message: string, response: Response) {
    super(message);
    this.name = "FetchError";
    this.response = response;
  }
}

export interface SDFError {
  message: string;
  code: number;
}

export class SDF {
  baseUrl: URL;
  wsBaseUrl: URL;
  currentToken?: string;
  update?: Update;

  constructor(config: Config) {
    this.baseUrl = config.sdfBaseUrl;
    this.wsBaseUrl = config.sdfBaseWsUrl;
  }

  async startUpdate() {
    if (!this.update) {
      this.setupUpdate();
    }
  }

  async setupUpdate() {
    const url = new URL(this.wsBaseUrl.toString());
    url.searchParams.set("token", `Bearer ${this.token}`);
    this.update = new Update(url.toString());
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
    return headers;
  }

  async list<T>(
    pathString: string,
    request?: IListRequest,
  ): Promise<IListReply<T>> {
    const args: Record<string, any> = { ...request };
    if (request?.query) {
      args["query"] = urlSafeBase64Encode(JSON.stringify(request.query));
    }
    return this.get(pathString, args);
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
    const url = new URL(`${basePath}/${requestPath}`, this.baseUrl);

    return url;
  }

  async get<T>(
    pathString: string,
    queryParams?: Record<string, any>,
  ): Promise<T> {
    const headers = this.standard_headers();

    const url = this.requestUrl(pathString);
    if (queryParams) {
      Object.keys(queryParams).forEach((key) =>
        url.searchParams.set(key, queryParams[key]),
      );
    }

    const request = new Request(url.toString(), {
      method: "GET",
      mode: "cors",
      headers,
    });
    const response: T = await this.send_request(request);
    return response;
  }

  async post<T>(pathString: string, args: Record<string, any>): Promise<T> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "POST",
      mode: "cors",
      headers,
      body: JSON.stringify(args),
    });
    const response: T = await this.send_request(request);
    return response;
  }

  async patch<T>(pathString: string, args: Record<string, any>): Promise<T> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "PATCH",
      mode: "cors",
      headers,
      body: JSON.stringify(args),
    });
    const response: T = await this.send_request(request);
    return response;
  }

  async delete<T>(pathString: string): Promise<T> {
    const headers = this.standard_headers();
    const url = this.requestUrl(pathString);
    const request = new Request(url.toString(), {
      method: "DELETE",
      mode: "cors",
      headers,
    });
    const response: T = await this.send_request(request);
    return response;
  }

  async send_request<T>(request: Request): Promise<T> {
    const response = await fetch(request);
    const responseJson: T = await response.json();
    return responseJson;
  }
}
