import _ from "lodash";
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

export class SDF {
  baseUrl: string;
  wsBaseUrl: string;
  currentToken?: string;
  update?: Update;

  constructor() {
    let baseUrl = process.env.VUE_APP_SDF || "http://localhost:5156";
    if (process.env.NODE_ENV === "production") {
      baseUrl = "https://api.systeminit.com";
    }
    this.baseUrl = baseUrl;

    let wsBaseUrl = process.env.VUE_APP_SDF_WS || "ws://localhost:5156/updates";
    if (process.env.NODE_ENV === "production") {
      wsBaseUrl = "https://api.systeminit.com/updates";
    }
    this.wsBaseUrl = wsBaseUrl;
  }

  async startUpdate() {
    if (!this.update) {
      this.setupUpdate();
    }
  }

  async setupUpdate() {
    const url = new URL(this.wsBaseUrl);
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
    let args: Record<string, any> = { ...request };
    if (request?.query) {
      args["query"] = urlSafeBase64Encode(JSON.stringify(request.query));
    }
    return this.get(pathString, args);
  }

  async get<T>(
    pathString: string,
    queryParams?: Record<string, any>,
  ): Promise<T> {
    let headers = this.standard_headers();

    const url = new URL(pathString, this.baseUrl);
    if (queryParams) {
      Object.keys(queryParams).forEach(key =>
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
    let headers = this.standard_headers();
    const url = new URL(pathString, this.baseUrl);
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
    let headers = this.standard_headers();
    const url = new URL(pathString, this.baseUrl);
    const request = new Request(url.toString(), {
      method: "PATCH",
      mode: "cors",
      headers,
      body: JSON.stringify(args),
    });
    const response: T = await this.send_request(request);
    return response;
  }

  async send_request<T>(request: Request): Promise<T> {
    let response = await fetch(request);
    if (!response.ok) {
      throw new FetchError("request failed", response);
    }
    let responseJson: T = await response.json();
    return responseJson;
  }
}

export const sdf = new SDF();
