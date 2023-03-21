import { createPinia } from "pinia";

import { piniaHooksPlugin, initPiniaApiToolkitPlugin } from "@si/vue-lib";
import api from "./api";

// initialize root pinia store/instance
const pinia = createPinia();

// init pinia plugins
// api request toolkit plugin - and pass in our axios instance
pinia.use(initPiniaApiToolkitPlugin({ api }));
pinia.use(piniaHooksPlugin);

export default pinia;
