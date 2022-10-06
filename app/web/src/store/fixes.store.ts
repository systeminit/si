import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useComponentsStore } from "@/store/components.store";
import promiseDelay from "@/utils/promise_delay";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { LoginResponse } from "@/service/session";

export type FixStatus = "success" | "failure" | "running" | "unstarted";

export type FixId = number;
type Fix = {
  id: FixId;
  name: string;
  recommendation: string;
  status: FixStatus;
};

export const useFixesStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspaceId;

  return addStoreHooks(
    defineStore(`w${workspaceId || "NONE"}/fixes`, {
      state: () => ({
        fixesById: {} as Record<FixId, Fix>,
        processedFixComponents: 0,
      }),
      getters: {
        allFixes(): Fix[] {
          return _.values(this.fixesById);
        },
        totalFixComponents() {
          const componentsStore = useComponentsStore();
          return componentsStore.allComponents.length;
        },
      },
      actions: {
        async LOAD_FIXES() {
          const componentsStore = useComponentsStore();

          if (
            !componentsStore.getRequestStatus("FETCH_COMPONENTS").value
              .isSuccess
          ) {
            await componentsStore.FETCH_COMPONENTS();
          }

          return new ApiRequest<LoginResponse>({
            method: "get",
            url: "/session/get_defaults",
            onSuccess: (response) => {
              this.populateMockFixes().then(() => {});
            },
          });
        },
        async EXECUTE_FIXES() {
          return new ApiRequest<LoginResponse>({
            method: "get",
            url: "/session/get_defaults",
            onSuccess: (response) => {
              this.executeMockFixes().then(() => {});
            },
          });
        },
        updateFix(fix: Fix) {
          this.fixesById[fix.id] = fix;
        },
        async populateMockFixes() {
          const componentsStore = useComponentsStore();

          for (const component of componentsStore.allComponents) {
            this.processedFixComponents += 1;

            if (["Region", "Docker Image"].includes(component.schemaName))
              continue;

            this.updateFix({
              id: component.id,
              name: _.sample(["This is a fix!", "Also a fix."]) ?? "",
              recommendation:
                _.sample([
                  "this is what we recommend you do - just fix this thing and you will be all good",
                  "honestly idk, you figure it out",
                ]) ?? "",
              status: "unstarted",
            });
            await promiseDelay(500);
          }
        },
        async executeMockFixes() {
          for (const fix of this.allFixes) {
            this.updateFix({
              ...fix,
              status: "running",
            });

            await promiseDelay(500);
          }

          await promiseDelay(1000);

          for (const fix of this.allFixes) {
            this.updateFix({
              ...fix,
              status: "success",
            });

            await promiseDelay(500);
          }
        },
      },
      async onActivated() {
        await this.LOAD_FIXES();
        console.log("Activated");
      },
    }),
  )();
};
