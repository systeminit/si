import Vue from "vue";
import Router from "vue-router";
import Home from "@/views/Home.vue";
import NotFound from "@/views/NotFound.vue";
import Callback from "@/components/Callback.vue";
import Workspaces from "@/views/Workspaces.vue";
import WorkspaceShow from "@/views/WorkspaceShow.vue";
import Integrations from "@/views/Integrations.vue";
import IntegrationShow from "@/views/IntegrationShow.vue";
import auth from "@/auth/authService";

Vue.use(Router);

const router = new Router({
  mode: "history",
  base: process.env.BASE_URL,
  routes: [
    {
      path: "/",
      name: "home",
      component: Home,
    },
    {
      path: "/profile",
      name: "profile",
      // route level code-splitting
      // this generates a separate chunk (about.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () =>
        import(/* webpackChunkName: "about" */ "./views/About.vue"),
    },
    {
      path: "/about",
      name: "about",
      // route level code-splitting
      // this generates a separate chunk (about.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () =>
        import(/* webpackChunkName: "about" */ "./views/About.vue"),
    },
    {
      path: "/callback",
      name: "callback",
      component: Callback,
    },
    {
      path: "/workspaces",
      name: "workspaces",
      component: Workspaces,
    },
    {
      path: "/workspaces/:id",
      name: "workspace",
      component: WorkspaceShow,
    },
    {
      path: "/integrations",
      name: "integrations",
      component: Integrations,
    },
    {
      path: "/integrations/:id",
      name: "integration",
      component: IntegrationShow,
    },
    {
      path: "*",
      component: NotFound,
    },
  ],
});

router.beforeEach((to, from, next) => {
  if (to.path == "/callback" || auth.isAuthenticated()) {
    return next();
  }

  auth.login({ target: to.fullPath });
});

export default router;
