import Axios, { AxiosResponse, InternalAxiosRequestConfig } from "axios";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { trackEvent } from "@/utils/tracking";

// api base url - can use a proxy or set a full url
let apiUrl: string;
if (import.meta.env.VITE_API_PROXY_PATH)
  apiUrl = `${window.location.origin}${import.meta.env.VITE_API_PROXY_PATH}`;
else if (import.meta.env.VITE_API_URL) apiUrl = import.meta.env.VITE_API_URL;
else throw new Error("Invalid API env var config");
export const API_HTTP_URL = apiUrl;

// set up websocket url, by replacing protocol and appending /ws
export const API_WS_URL = `${API_HTTP_URL.replace(/^http/, "ws")}/ws`;

export const sdfApiInstance = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: API_HTTP_URL,
});

function injectBearerTokenAuth(config: InternalAxiosRequestConfig) {
  // inject auth token from the store as a custom header
  const authStore = useAuthStore();
  const workspacesStore = useWorkspacesStore();

  config.headers = config.headers || {};
  if (authStore.token) {
    config.headers.authorization = `Bearer ${authStore.token}`;
  }
  // automatically set selected workspace pk header
  // we will probably want to do something similar with change-set
  // also need to remove workspace pk from body params in many places
  if (workspacesStore.urlSelectedWorkspaceId) {
    config.headers.WorkspacePk = workspacesStore.urlSelectedWorkspaceId;
  }
  return config;
}

sdfApiInstance.interceptors.request.use(injectBearerTokenAuth);

async function handleForcedChangesetRedirection(response: AxiosResponse) {
  if (response.headers.force_changeset_pk) {
    const changeSetsStore = useChangeSetsStore();
    await changeSetsStore.setActiveChangeset(
      response.headers.force_changeset_pk,
    );
  }

  return response;
}

async function handleProxyTimeouts(response: AxiosResponse) {
  // some weird timeouts happening and triggering nginx 404 when running via the CLI
  // here we will try to detect them, track it, and give user some help
  if (
    response.status === 404 &&
    response.headers?.["content-type"] !== "application/json"
  ) {
    trackEvent("api_404_timeout");
    // redirect to oops page after short timeout so we give tracker a chance to send event
    setTimeout(() => {
      if (window) window.location.href = "/oops";
    }, 500);
  }
  return response;
}

sdfApiInstance.interceptors.response.use(handleProxyTimeouts);
sdfApiInstance.interceptors.response.use(handleForcedChangesetRedirection);

export const authApiInstance = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: import.meta.env.VITE_AUTH_API_URL,
  withCredentials: true, // needed to attach the cookie
});

export const moduleIndexApiInstance = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: import.meta.env.VITE_MODULE_INDEX_API_URL,
});
moduleIndexApiInstance.interceptors.request.use(injectBearerTokenAuth);
