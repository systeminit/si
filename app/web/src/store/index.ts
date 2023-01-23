import { createPinia } from "pinia";

import { initPiniaApiToolkitPlugin } from "./lib/pinia_api_tools";
import { piniaHooksPlugin } from "./lib/pinia_hooks_plugin";
import api from "./api";

// initialize root pinia store/instance
const pinia = createPinia();

// init pinia plugins
// api request toolkit plugin - and pass in our axios instance
pinia.use(initPiniaApiToolkitPlugin({ api }));
pinia.use(piniaHooksPlugin);

export default pinia;
