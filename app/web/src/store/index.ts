import { createPinia } from "pinia";

import { initPiniaApiToolkitPlugin } from "@/utils/pinia_api_tools";
import api from "@/utils/api";
import { piniaHooksPlugin } from "@/utils/pinia_hooks_plugin";

// initialize root pinia store/instance
const pinia = createPinia();

// init pinia plugins
// api request toolkit plugin - and pass in our axios instance
pinia.use(initPiniaApiToolkitPlugin({ api }));
pinia.use(piniaHooksPlugin);

export default pinia;
