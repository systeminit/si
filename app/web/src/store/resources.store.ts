import { defineStore } from "pinia";
import _ from "lodash";
import { ApiRequest } from "@/utils/pinia_api_tools";

import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { useRealtimeStore } from "./realtime/realtime.store";
import { ComponentId, useComponentsStore } from "./components.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useFixesStore } from "./fixes/fixes.store";

export type ResourceId = number;

export type MockResource = {
  id: ResourceId;
  componentId: ComponentId;
  name: string;
  kind: string;
  health: ResourceHealth;
  status: ResourceStatus;
  confirmations: Confirmation[];
};

// TODO: need to review this... clarify "resource health" vs "resource status"
// and should probably align closer to "qualification status"
export type ConfirmationResult = "success" | "failure" | "running";
export enum ResourceHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
  Running = "running",
}

export enum ResourceStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface Confirmation {
  title: string;
  result: ConfirmationResult;
  health: ResourceHealth;
  link?: string;
  description?: string;
  output?: string[];
}

export enum ComponentSchema {
  DockerImage = "Docker Image",
  DockerHubCredential = "Docker Hub Credential",
  KubernetesNamespace = "Kubernetes Namespace",
  KubernetesDeployment = "Kubernetes Deployment",
  CoreOsButane = "Butane",
}

export interface ResourceSummaryForComponent {
  id: number;
  name: string;
  health: ResourceHealth;
  schema: string;
  resource?: MockResource;
}

type ConfirmationStats = Record<ConfirmationResult | "total", number>;

export const useResourcesStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspaceId;
  return addStoreHooks(
    defineStore(`w${workspaceId}/resources`, {
      state: () => ({
        // resourcesByComponentId: {} as Record<ComponentId, MockResource>,
        selectedResourceId: null as ResourceId | null,
      }),
      getters: {
        resourcesByComponentId(): Record<ComponentId, MockResource> {
          const componentsStore = useComponentsStore();
          const fixesStore = useFixesStore();

          const resources = _.map(
            componentsStore.allComponents,
            (component) => {
              const fix = fixesStore.fixesByComponentId[component.id];
              // no fix (in our mock setup) means the component never needs to be created
              const resourceExists = !fix || fix?.status === "success";

              const confirmationResult: ConfirmationResult = resourceExists
                ? "success"
                : "failure";

              const health = resourceExists
                ? ResourceHealth.Ok
                : ResourceHealth.Error;
              const status = resourceExists
                ? ResourceStatus.Created
                : ResourceStatus.Pending;

              const resource: MockResource = {
                id: 5000 + component.id,
                componentId: component.id,
                name: component.displayName,
                kind: component.schemaName,
                health,
                status,
                confirmations: [
                  {
                    title: "Does The Resource Exist?",
                    result: confirmationResult,
                    health,
                    description: resourceExists
                      ? "Checks if the resource actually exists. This resource exists!"
                      : "Checks if the resource actually exists. This resource has not been created yet. Please run the fix above to create it!",
                  },
                ],
              };
              return resource;
            },
          );
          return _.keyBy(resources, (r) => r.componentId);
        },
        allResources(): MockResource[] {
          return _.values(this.resourcesByComponentId);
        },
        selectedResource(): MockResource | null {
          const componentsStore = useComponentsStore();
          return componentsStore.selectedComponentId
            ? this.resourcesByComponentId[componentsStore.selectedComponentId]
            : null;
        },

        // confirmations living here for now... might move into their own store later?
        confirmationsByComponentId(): Record<ComponentId, Confirmation[]> {
          return _.mapValues(
            this.resourcesByComponentId,
            (c) => c.confirmations,
          );
        },
        confirmationStatsByComponentId(): Record<
          ComponentId,
          ConfirmationStats
        > {
          return _.mapValues(
            this.confirmationsByComponentId,
            (confirmations, _componentId) => {
              const grouped = _.groupBy(confirmations, (c) => {
                if (c.health === ResourceHealth.Ok) return "success";
                if (c.health === ResourceHealth.Error) return "failure";
                // TODO: fix this when we align with qualificaitons
                if (c.health === ResourceHealth.Warning) return "failure";
                return "running";
              });
              return {
                failure: grouped.failure?.length || 0,
                success: grouped.success?.length || 0,
                running: grouped.running?.length || 0,
                total: confirmations.length || 0,
              };
            },
          );
        },

        // single status per component
        confirmationResultByComponentId(): Record<
          ComponentId,
          ConfirmationResult
        > {
          return _.mapValues(this.confirmationStatsByComponentId, (cs) => {
            if (cs.running) return "running";
            if (cs.failure > 0) return "failure";
            return "success";
          });
        },

        // stats/totals by component
        componentsConfirmationStats(): ConfirmationStats {
          const grouped = _.groupBy(this.confirmationResultByComponentId);
          return {
            failure: grouped.failure?.length || 0,
            success: grouped.success?.length || 0,
            running: grouped.running?.length || 0,
            total: _.keys(this.confirmationResultByComponentId).length,
          };
        },

        // roll up to single confirmations result for the workspace
        workspaceConfirmationResult(): ConfirmationResult {
          if (this.componentsConfirmationStats.running > 0) return "running";
          if (this.componentsConfirmationStats.failure > 0) return "failure";
          return "success";
        },
      },
      actions: {
        // actually fetches diagram-style data, but we have a computed getter to turn back into more generic component data above
        async FETCH_RESOURCES_LIST() {
          return new ApiRequest<{ components: ResourceSummaryForComponent[] }>({
            method: "get",
            url: "resource/list_resources_by_component",
            params: {
              // resources should only be fetched for head, but this was being passed in...
              visibility_change_set_pk: -1,
            },
            onSuccess: (response) => {
              // we'll hit the resources endpoint, but for now the resources are populated with fake data already (see getters)
            },
          });
        },

        // for now we'll populate the resources using data in the fixes store (which is mocked)
        // but it may turn out the fixes store and resources store are combined?
        // more needs to be worked out about what the shape of the data and backend will look like...
        populateMockResources() {},
      },
      onActivated() {
        this.FETCH_RESOURCES_LIST();

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
          {
            eventType: "ChangeSetApplied",
            callback: () => {
              this.FETCH_RESOURCES_LIST();
            },
          },
          // TODO: should just push updated resource data directly
        ]);

        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
