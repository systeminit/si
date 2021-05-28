<template>
  <div class="flex">
    <div
      class="flex items-center text-xs font-light text-gray-300"
      v-if="services"
    >
      Services:
      <div class="ml-1">
        <span class="green" v-if="services.ok">{{ services.ok }}</span>
        <span v-if="services.ok && services.error">/</span>
        <span class="red" v-if="services.error">{{ services.error }}</span>
      </div>
    </div>
    <div
      class="flex items-center ml-2 text-xs font-light text-gray-300"
      v-if="computingResources"
    >
      Computing Resources:
      <div class="ml-1">
        <span class="green" v-if="computingResources.ok">{{
          computingResources.ok
        }}</span>
        <span v-if="computingResources.ok && computingResources.error">/</span>
        <span class="red" v-if="computingResources.error">{{
          computingResources.error
        }}</span>
      </div>
    </div>
    <div
      class="flex items-center ml-2 text-xs font-light text-gray-300"
      v-if="providers"
    >
      Providers:
      <div class="ml-1">
        <span class="green" v-if="providers.ok">{{ providers.ok }}</span>
        <span v-if="providers.ok && providers.error">/</span>
        <span class="red" v-if="providers.error">{{ providers.error }}</span>
      </div>
    </div>
    <div
      class="flex items-center ml-2 text-xs font-light text-gray-300"
      v-if="changes"
    >
      Changes:
      <div class="ml-1 mr-1 green" v-if="changes.newNodes != undefined">
        +{{ changes.newNodes }} &nbsp;
      </div>

      <div class="mr-1 red" v-if="changes.deletedNodes != undefined">
        -{{ changes.deletedNodes }} &nbsp;
      </div>

      <div class="updates" v-if="changes.modifiedNodes != undefined">
        u{{ changes.modifiedNodes }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import {
  workspace$,
  changeSet$,
  refreshChangesSummary$,
  resources$,
  system$,
  refreshResourceSummary$,
} from "@/observables";
import { combineLatest } from "rxjs";
import { tap, pluck } from "rxjs/operators";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import {
  IChangesSummaryRequest,
  ApplicationDal,
  ResourceSummaryKind,
} from "@/api/sdf/dal/applicationDal";
import { ResourceInternalHealth, Resource } from "si-entity";

interface Data {
  changes: {
    newNodes?: number;
    deletedNodes?: number;
    modifiedNodes?: number;
  } | null;
  servicesCache: Resource[];
  computingResourcesCache: Resource[];
  providersCache: Resource[];
}

export type ResourceStatus = {
  ok?: number;
  error?: number;
} | null;

export default Vue.extend({
  name: "MenuSummary",
  props: {
    applicationId: {
      type: String,
    },
  },
  data(): Data {
    return {
      changes: null,
      servicesCache: [],
      computingResourcesCache: [],
      providersCache: [],
    };
  },
  computed: {
    services(): ResourceStatus {
      let ok: number | undefined = 0;
      let error: number | undefined = 0;
      for (const r of this.servicesCache) {
        if (r.internalHealth == ResourceInternalHealth.Ok) {
          ok++;
        } else {
          error++;
        }
      }
      if (ok == 0) {
        ok = undefined;
      }
      if (error == 0) {
        error = undefined;
      }

      if (ok == undefined && error == undefined) {
        return null;
      } else {
        return { ok, error };
      }
    },
    computingResources(): ResourceStatus {
      let ok: number | undefined = 0;
      let error: number | undefined = 0;
      for (const r of this.computingResourcesCache) {
        if (r.internalHealth == ResourceInternalHealth.Ok) {
          ok++;
        } else {
          error++;
        }
      }
      if (ok == 0) {
        ok = undefined;
      }
      if (error == 0) {
        error = undefined;
      }

      if (ok == undefined && error == undefined) {
        return null;
      } else {
        return { ok, error };
      }
    },
    providers(): ResourceStatus {
      let ok: number | undefined = 0;
      let error: number | undefined = 0;
      for (const r of this.providersCache) {
        if (r.internalHealth == ResourceInternalHealth.Ok) {
          ok++;
        } else {
          error++;
        }
      }
      if (ok == 0) {
        ok = undefined;
      }
      if (error == 0) {
        error = undefined;
      }

      if (ok == undefined && error == undefined) {
        return null;
      } else {
        return { ok, error };
      }
    },
  },
  subscriptions(): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));
    return {
      changeSet: changeSet$.pipe(
        tap(c => {
          if (c == null) {
            //@ts-ignore
            this.changes = null;
          }
        }),
      ),
      loadChangeSetData: combineLatest(
        applicationId$,
        workspace$,
        changeSet$,
        refreshChangesSummary$,
      ).pipe(
        tap(async ([applicationId, workspace, changeSet]) => {
          if (applicationId && workspace) {
            let request: IChangesSummaryRequest = {
              applicationId,
              workspaceId: workspace.id,
            };
            if (changeSet) {
              request["changeSetId"] = changeSet.id;
            }
            let reply = await ApplicationDal.changesSummary(request);
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              if (reply.currentChangeSet) {
                // @ts-ignore
                this.changes = reply.currentChangeSet;
              }
            }
          }
        }),
      ),
      resources: resources$.pipe(
        tap(r => {
          let isUpdated = false;
          if (r.entityType == "service") {
            //@ts-ignore
            for (let x = 0; x < this.servicesCache.length; x++) {
              // @ts-ignore
              if (r.id == this.servicesCache[x].id) {
                isUpdated = true;
                //@ts-ignore
                Vue.set(this.servicesCache, x, r);
              }
              if (!isUpdated) {
                //@ts-ignore
                this.servicesCache.push(r);
              }
            }
          } else if (r.entityType == "kubernetesCluster") {
            //@ts-ignore
            for (let x = 0; x < this.computingResourcesCache.length; x++) {
              // @ts-ignore
              if (r.id == this.computingResourcesCache[x].id) {
                isUpdated = true;
                //@ts-ignore
                Vue.set(this.computingResourcesCache, x, r);
              }
              if (!isUpdated) {
                //@ts-ignore
                this.computingResourcesCache.push(r);
              }
            }
          } else if (r.entityType == "cloudProvider") {
            //@ts-ignore
            for (let x = 0; x < this.providersCache.length; x++) {
              // @ts-ignore
              if (r.id == this.providersCache[x].id) {
                isUpdated = true;
                //@ts-ignore
                Vue.set(this.providersCache, x, r);
              }
              if (!isUpdated) {
                //@ts-ignore
                this.providersCache.push(r);
              }
            }
          }
        }),
      ),
      updateResources: combineLatest(
        applicationId$,
        workspace$,
        system$,
        refreshResourceSummary$,
      ).pipe(
        tap(async ([applicationId, workspace, system]) => {
          if (applicationId && workspace && system) {
            let reply = await ApplicationDal.resourceSummary({
              // @ts-ignore
              applicationId,
              workspaceId: workspace.id,
              systemId: system.id,
              kind: ResourceSummaryKind.Service,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              //@ts-ignore
              this.servicesCache = reply.resources;
            }
            let computeReply = await ApplicationDal.resourceSummary({
              // @ts-ignore
              applicationId,
              workspaceId: workspace.id,
              systemId: system.id,
              kind: ResourceSummaryKind.ComputingResources,
            });
            if (computeReply.error) {
              emitEditorErrorMessage(computeReply.error.message);
            } else {
              //@ts-ignore
              this.computingResourcesCache = computeReply.resources;
            }
            let providerReply = await ApplicationDal.resourceSummary({
              // @ts-ignore
              applicationId,
              workspaceId: workspace.id,
              systemId: system.id,
              kind: ResourceSummaryKind.Providers,
            });
            if (providerReply.error) {
              emitEditorErrorMessage(providerReply.error.message);
            } else {
              //@ts-ignore
              this.providersCache = providerReply.resources;
            }
          }
        }),
      ),
    };
  },
});
</script>

<style scoped>
.green {
  color: #a6e2a5;
}

.additions {
  color: #a6e2a5;
}

.removals {
  color: #e2a5a5;
}

.updates {
  color: #e2c8a5;
}

.red {
  color: #e2a5a5;
}
</style>
