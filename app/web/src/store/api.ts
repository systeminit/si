import Axios from "axios";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";

// api base url - can use a proxy or set a full url
let apiUrl: string;
if (import.meta.env.VITE_API_PROXY_PATH)
  apiUrl = `${window.location.origin}${import.meta.env.VITE_API_PROXY_PATH}`;
else if (import.meta.env.VITE_API_URL) apiUrl = import.meta.env.VITE_API_URL;
else throw new Error("Invalid API env var config");
export const API_HTTP_URL = apiUrl;

// set up websocket url, by replacing protocol and appending /ws
export const API_WS_URL = `${API_HTTP_URL.replace(/^http/, "ws")}/ws`;

const api = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: API_HTTP_URL,
});

// add axios interceptors to add auth headers, handle logout errors, etc...
api.interceptors.request.use((config) => {
  // inject auth token from the store as a custom header
  const authStore = useAuthStore();
  const workspacesStore = useWorkspacesStore();

  config.headers = config.headers || {};
  if (authStore.token) {
    config.headers.authorization = `Bearer ${authStore.token}`;
  }
  // automatically set selected workspace id header
  // we will probably want to do something similar with change-set
  // also need to remove workspace id from body params in many places
  if (workspacesStore.selectedWorkspaceId) {
    config.headers.WorkspaceId = workspacesStore.selectedWorkspaceId;
  }
  return config;
});

export default api;
