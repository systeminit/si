export interface Config {
  routerBase: string | undefined;
  sdfBaseUrl: URL;
  sdfBaseWsUrl: URL;
}

export const config: Config = {
  routerBase: process.env.BASE_URL,
  sdfBaseUrl: new URL(
    process.env.VUE_APP_SDF_BASE_HTTP_URL ||
      (process.env.NODE_ENV == "production"
        ? "https://app.systeminit.com/api"
        : "http://localhost:8080/api"),
  ),
  sdfBaseWsUrl: new URL(
    process.env.VUE_APP_SDF_BASE_WS_URL ||
      (process.env.NODE_ENV == "production"
        ? "wss://app.systeminit.com/api/updates"
        : "ws://localhost:8080/api/updates"),
  ),
};
