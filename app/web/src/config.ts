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
        ? `https://${window.location.origin}/api`
        : `${window.location.origin}/api`),
  ),
  sdfBaseWsUrl: new URL(
    process.env.VUE_APP_SDF_BASE_WS_URL ||
      (process.env.NODE_ENV == "production"
        ? `wss://${window.location.host}/api/ws/billing_account_updates`
        : `ws://${window.location.host}/api/ws/billing_account_updates`),
  ),
};
