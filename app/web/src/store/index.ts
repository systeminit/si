import { createPinia } from "pinia";

import { piniaHooksPlugin, initPiniaApiToolkitPlugin, registerApi } from "@si/vue-lib/pinia";
import { sdfApiInstance, authApiInstance, moduleIndexApiInstance } from "./apis.web";

// initialize root pinia store/instance
const pinia = createPinia();

// init pinia plugins
// api request toolkit plugin - and pass in our axios instance
const piniaApiToolkitPlugin = initPiniaApiToolkitPlugin({
  api: sdfApiInstance,
});
export const SdfApiRequest = registerApi(sdfApiInstance);
export const AuthApiRequest = registerApi(authApiInstance);
export const ModuleIndexApiRequest = registerApi(moduleIndexApiInstance);

pinia.use(piniaApiToolkitPlugin);
pinia.use(piniaHooksPlugin);

export default pinia;
