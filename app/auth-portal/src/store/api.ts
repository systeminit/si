import Axios from "axios";
// import { useAuthStore } from "./auth.store";

// api base url - can use a proxy or set a full url
let apiUrl: string;
if (import.meta.env.VITE_API_PROXY_PATH)
  apiUrl = `${window.location.origin}${import.meta.env.VITE_API_PROXY_PATH}`;
else if (import.meta.env.VITE_AUTH_API_URL)
  apiUrl = import.meta.env.VITE_AUTH_API_URL;
else throw new Error("Invalid API env var config");
export const API_HTTP_URL = apiUrl;

const api = Axios.create({
  headers: {
    "Content-Type": "application/json",
  },
  withCredentials: true,
  baseURL: API_HTTP_URL,
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
if (window) (window as any).api = api;

// // add axios interceptors to add auth headers, handle logout errors, etc...
// api.interceptors.request.use((config) => {
//   // inject auth token from the store as a custom header
//   const authStore = useAuthStore();

//   config.headers = config.headers || {};
//   if (authStore.token) {
//     config.headers.authorization = `Bearer ${authStore.token}`;
//   }
//   return config;
// });

export default api;
