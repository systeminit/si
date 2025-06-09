import * as _ from "lodash-es";
import Axios, {
  AxiosError,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from "axios";
import { useToast } from "vue-toastification";
import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { trackEvent } from "@/utils/tracking";
import FiveHundredError from "@/components/toasts/FiveHundredError.vue";
import MaintenanceMode from "@/components/toasts/MaintenanceMode.vue";
import UnscheduledDowntime from "@/components/toasts/UnscheduledDowntime.vue";

// api base url - can use a proxy or set a full url
let apiUrl: string;
if (import.meta.env.VITE_API_PROXY_PATH)
  apiUrl = `${window.location.origin}${import.meta.env.VITE_API_PROXY_PATH}`;
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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function injectBearerTokenAuth(config: InternalAxiosRequestConfig) {
  // inject auth token from the store as a custom header
  const authStore = useAuthStore();
  config.headers = config.headers || {};

  const token = authStore.selectedOrDefaultAuthToken;
  if (token) {
    config.headers.authorization = `Bearer ${token}`;
  }
  return config;
}

sdfApiInstance.interceptors.request.use(injectBearerTokenAuth);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
if (typeof window !== "undefined") (window as any).sdf = sdfApiInstance;

async function handleForcedChangesetRedirection(response: AxiosResponse) {
  if (response.headers.force_change_set_id) {
    const changeSetsStore = useChangeSetsStore();

    await changeSetsStore.setActiveChangeset(
      response.headers.force_change_set_id,
      true,
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
      if (typeof window !== "undefined") window.location.href = "/oops";
    }, 500);
  }
  return response;
}

async function handle500(error: AxiosError) {
  const toast = useToast();
  if (error?.response?.status === 500) {
    toast(
      {
        component: FiveHundredError,
        props: {
          requestUrl: error?.config?.url,
          message: error?.response?.data,
        },
      },
      {
        timeout: false,
      },
    );
  }
  return Promise.reject(error);
}

async function handleOutageModes(error: AxiosError) {
  if (error?.response?.status === 503) {
    const toast = useToast();
    toast(
      {
        id: "MAINTENANCE",
        component: MaintenanceMode,
      },
      {
        timeout: 15000,
        hideProgressBar: false,
      },
    );
  } else if (
    error?.response?.status === 502 ||
    error?.response?.status === 504
  ) {
    const toast = useToast();
    toast(
      {
        id: "DOWNTIMETOAST",
        component: UnscheduledDowntime,
      },
      {
        timeout: 15000,
        hideProgressBar: false,
      },
    );
  }
  return Promise.reject(error);
}

sdfApiInstance.interceptors.response.use(handleProxyTimeouts, handle500);
sdfApiInstance.interceptors.response.use(
  handleForcedChangesetRedirection,
  handleOutageModes,
);

export const authApiInstance = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: import.meta.env.VITE_AUTH_API_URL,
  withCredentials: true, // needed to attach the cookie
});
authApiInstance.interceptors.request.use(injectBearerTokenAuth);

export const moduleIndexApiInstance = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  baseURL: import.meta.env.VITE_MODULE_INDEX_API_URL,
});
moduleIndexApiInstance.interceptors.request.use(injectBearerTokenAuth);
