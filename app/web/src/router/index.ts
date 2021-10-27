// import _ from "lodash";
// import {
//   ISessionDalIsAuthenticatedReply,
//   SessionDal,
// } from "@/api/sdf/dal/sessionDal";
// import { SDFError } from "@/api/sdf";
// import { billingAccount$, user$ } from "@/observables";
// import { combineLatest, from as fromRx } from "rxjs";
// import { switchMap, take } from "rxjs/operators";

import routes from "./routes";
import { config } from "@/config";
import { createWebHistory, createRouter } from "vue-router";

// export const routeCheck = async (to: any, from: any, next: any) => {
//   // 1. Is the user asking to sign in? If so, route there.
//   // 2. Is the user authenticated? If not, route there.
//   // 3. Is the system loaded? If not, route to the loading screen, then redirect to the URL the user asked for.
//   // 4. Route to the requested location.
//   //
//   if (to.path == "/authenticate/signup") {
//     return next();
//   } else if (to.path == "/authenticate/login") {
//     return next();
//   }

//   let reply = (await combineLatest([user$, billingAccount$])
//     .pipe(
//       switchMap(([user, billingAccount]) => {
//         return fromRx(SessionDal.isAuthenticated({ user, billingAccount }));
//       }),
//       take(1),
//     )
//     .toPromise()) as ISessionDalIsAuthenticatedReply;

//   let authenticated: boolean | SDFError = false;
//   if (reply.error) {
//     authenticated = reply.error;
//   } else if (reply.logout) {
//     await SessionDal.logout();
//     authenticated = false;
//   } else if (reply.login) {
//     authenticated = false;
//   } else {
//     user$.next(reply.user);
//     billingAccount$.next(reply.billingAccount);
//     authenticated = true;
//   }

//   // @ts-ignore
//   if (authenticated === false || authenticated.error) {
//     // @ts-ignore
//     if (authenticated.error) {
//       console.log("Error checking authentication", authenticated);
//     }
//     return next("/authenticate/login");
//   } else {
//     return next();
//   }
// };

const router = createRouter({
  history: createWebHistory(config.routerBase),
  routes,
});

// router.beforeEach(async (to, from, next) => {
//   // await routeCheck(to, from, next);
// });

export default router;
