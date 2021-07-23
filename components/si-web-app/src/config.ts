export interface Config {
  routerBase: string | undefined;
  sdfBaseUrl: URL;
  sdfBaseWsUrl: URL;
}

export const config: Config = {
  routerBase: process.env.BASE_URL,
  sdfBaseUrl: new URL(
    process.env.VUE_APP_SDF_BASE_URL ||
      (process.env.NODE_ENV == "production"
        ? "https://api.systeminit.com/api"
        : "http://localhost:5156"),
  ),
  sdfBaseWsUrl: new URL(
    process.env.VUE_APP_SDF_BASE_WS_URL ||
      (process.env.NODE_ENV == "production"
        ? "ws://api.systeminit.com/api/updates"
        : "ws://localhost:5156/updates"),
  ),
};
