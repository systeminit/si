import routes from "./routes";
import { config } from "@/config";
import { createWebHistory, createRouter } from "vue-router";
import { SessionService } from "@/api/sdf/service/session";

export const routeCheck = async (to: any, _from: any, next: any) => {
  if (to.path == "/authenticate/signup" || to.path == "/authenticate/login") {
    return next();
  }

  const authenticated = await SessionService.isAuthenticated();
  if (authenticated === false || authenticated.error) {
    if (authenticated.error) {
      console.log("Error checking authentication", authenticated);
    }
    return next("/authenticate/login");
  } else {
    return next();
  }
};

const router = createRouter({
  history: createWebHistory(config.routerBase),
  routes,
});

router.beforeEach(async (to, from, next) => {
  await routeCheck(to, from, next);
});

export default router;
