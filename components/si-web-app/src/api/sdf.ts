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
  token?: string;

  constructor() {
    let baseUrl = process.env.VUE_APP_SDF || "http://localhost:5156";
    if (process.env.NODE_ENV === "production") {
      baseUrl = "https://api.systeminit.com";
    }
    this.baseUrl = baseUrl;
  }

  async post<T>(pathString: string, args: Record<string, any>): Promise<T> {
    const url = new URL(pathString, this.baseUrl);
    const headers = new Headers();
    headers.set("Content-Type", "application/json");
    if (this.token) {
      headers.set("Authorization", `Bearer ${this.token}`);
    }
    const request = new Request(url.toString(), {
      method: "POST",
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
