import * as _ from "lodash-es";
import Axios, {
  InternalAxiosRequestConfig,
} from "axios";

// api base url - can use a proxy or set a full url
let apiUrl: string;
if (import.meta.env.VITE_API_PROXY_PATH) {
    apiUrl = `http://localhost:8080${import.meta.env.VITE_API_PROXY_PATH}`;
}
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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function injectBearerTokenAuth(config: InternalAxiosRequestConfig) {
  // inject auth token from the store as a custom header
  config.headers = config.headers || {};

  const token = `eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX3BrIjoiMDFIUkZFVjBSTVdNSDVTR0JHREFSSDNHNDgiLCJ3b3Jrc3BhY2VfcGsiOiIwMUhSRkVWMFMyM1IxRzIzUlA3NVFRRENBNyIsImlhdCI6MTc0MTAxMjUxMiwic3ViIjoiMDFIUkZFVjBSTVdNSDVTR0JHREFSSDNHNDgifQ.Om96vxeFVbeTebsc5ECkbPvtzt3tg_6YeiDM6QZ9H5Kzp4-IWK0vKXbF6hPCbornp-8IuUZBUowPg6BzXgqsvePzqoud8x6S01kzO2AmDCHTPdSprfT1hF34KkBDBVnCAS-hbz5w7iwWoVyfX5goRkODgfWXhjxSgsmHmjEwoYFGC7dUbZvl7h2M3wyMcr8Ls6CTi3FAuOmPZ0ld1mKg3323C8pPEkCt4e15AZ0dHAyWxVu8MioJjpf7XB_pQt_GT3MTznGOtb7OWnV3jmhGyVKhgM95hycRImWmT39YZrnwroNkosZFFChsm8fLQDs9u1b2Ap227RWd1rXWtMXIREaH6Qp_mWQURbC0HJs8CtS8z05CvhtLjqbQQ1S6P1ybjYPMD-bToPeFzNJd2UbNHR1eGRF0Yl4OZfnOf0O97r2H5CrvtjDYlA8PKma1GcBlk24NStfnPoiZmfzhPNZuYuYL6E2DEwM6GXUajTPQmvHavujYwV1ziy9-H8CiVHpa4TBW4UxVDyBI85k4RBXqvYBRJb1Sf6bS09DVWXMS5u01BHIsX9GgFl3t0L_0BkElseOWQL8s-5ARBXVi1o-O7qZ1Eph7mRkuxbl1d4Asym1j8PbrsQFfw3waUL_UOrhvDcKDMkG2Cxmj1cZr6QJSK668AdFDkEWe4lx84TCkKx8`;
  if (token) {
    config.headers.authorization = `Bearer ${token}`;
  }
  return config;
}

sdfApiInstance.interceptors.request.use(injectBearerTokenAuth);

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
