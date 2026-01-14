import { defineStore } from "pinia";
import { ApiRequest } from "@si/vue-lib/pinia";
import { useRealtimeStore } from "./realtime/realtime.store";

export const useSystemStatusStore = () => {
  const realtimeStore = useRealtimeStore();

  return defineStore("system-status", {
    state: () => ({
      // returns shorthand git sha from API
      apiGitBranch: null as string | null, // not supported on backend yet...
      apiGitSha: null as string | null, // TODO: currently set up to only work in dev mode, but probably always want this available

      // TODO: ideally these would be injected into import.meta?
      // @ts-ignore
      webGitBranch: __VITE_GIT_BRANCH__,
      // @ts-ignore
      webGitSha: __VITE_GIT_SHA__,
    }),
    getters: {},
    actions: {
      async CHECK_CURRENT_API_VERSION() {
        // does not actually return branch yet
        return new ApiRequest<{ sha: string; branch: string }>({
          url: "/dev/get_current_git_sha",
          onSuccess: (response) => {
            this.apiGitSha = response.sha;
            this.apiGitBranch = response.branch || "main";
          },
        });
      },

      registerRequestsBegin(requestUlid: string, actionName: string) {
        realtimeStore.inflightRequests.set(requestUlid, actionName);
      },
      registerRequestsEnd(requestUlid: string) {
        realtimeStore.inflightRequests.delete(requestUlid);
      },
    },
  })();
};
