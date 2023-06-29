import Axios, { AxiosRequestConfig, AxiosResponse } from "axios";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";

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

function injectBearerTokenAuth(config: AxiosRequestConfig) {
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
  if (workspacesStore.selectedWorkspacePk) {
    config.headers.WorkspacePk = workspacesStore.selectedWorkspacePk;
  }
  return config;
}

sdfApiInstance.interceptors.request.use(injectBearerTokenAuth);

async function handleForcedChangesetRedirection(response: AxiosResponse) {
  const setActiveChangeset = useChangeSetsStore().setActiveChangeset;

  // TODO(victor) I made this a field with a prefix just to make it work and test the experience
  // It probably makes sense to make this come as a header from the backend so only this interceptor needs
  // to care about it
  if (response.headers.force_changeset_pk) {
    await setActiveChangeset(response.headers.force_changeset_pk);
  }

  return response;
}

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
