<template>
  <SummaryCard>
    <template v-slot:title>Computing Resources</template>

    <template v-slot:content>
      <div class="flex flex-col w-full h-full">
        <div class="flex flex-row flex-wrap w-full h-full mx-1">
          <div
            class="mr-2"
            v-for="resource in computingResourcesData"
            :key="resource.id"
          >
            <ResourceVisualization :resource="resource" />
          </div>
        </div>

        <div class="flex justify-end mt-2" v-show="showButton">
          <div class="flex items-center justify-center button">
            <div class="mx-1 align-middle button-text">Sync</div>
          </div>
        </div>
      </div>
    </template>
  </SummaryCard>
</template>

<script lang="ts">
import Vue from "vue";

import ResourceVisualization from "@/molecules/ComputingResourceSummary/ComputingResourceVisualization.vue";

import SummaryCard from "@/atoms/SummaryCard.vue";
import {
  IResourceSummaryReplySuccess,
  ApplicationDal,
  ResourceSummaryKind,
} from "@/api/sdf/dal/applicationDal";
import { combineLatest } from "rxjs";
import { pluck, tap, debounceTime } from "rxjs/operators";
import {
  workspace$,
  system$,
  resources$,
  refreshResourceSummary$,
} from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  computingResourcesData: IResourceSummaryReplySuccess["resources"];
}

export default Vue.extend({
  name: "ComputingResourceSummary",
  components: {
    ResourceVisualization,
    SummaryCard,
  },
  props: {
    showButton: {
      type: Boolean,
      default: false,
    },
    applicationId: {
      type: String,
    },
  },
  data(): IData {
    return {
      computingResourcesData: [],
    };
  },
  subscriptions(): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));

    return {
      system: system$,
      workspace: workspace$,
      resources: resources$.pipe(
        debounceTime(5000),
        tap(async r => {
          //@ts-ignore
          if (this.applicationId && this.system && this.workspace) {
            let reply = await ApplicationDal.resourceSummary({
              //@ts-ignore
              applicationId: this.applicationId,
              //@ts-ignore
              workspaceId: this.workspace.id,
              //@ts-ignore
              systemId: this.system.id,
              kind: ResourceSummaryKind.ComputingResources,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              //@ts-ignore
              this.computingResourcesData = reply.resources;
            }
          }

          //let isUpdated = false;
          //// @ts-ignore
          //if (this.computingResourcesData) {
          //  // @ts-ignore
          //  for (let x = 0; x < this.computingResourcesData.length; x++) {
          //    // @ts-ignore
          //    if (r.id == this.computingResourcesData[x].id) {
          //      isUpdated = true;
          //      // @ts-ignore
          //      Vue.set(this.computingResourcesData, x, r);
          //    }
          //  }
          //  if (!isUpdated) {
          //    if (r.entityType == "kubernetesCluster") {
          //      // @ts-ignore
          //      this.computingResourcesData.push(r);
          //    }
          //  }
          //}
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
              applicationId,
              workspaceId: workspace.id,
              systemId: system.id,
              kind: ResourceSummaryKind.ComputingResources,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              // @ts-ignore
              this.computingResourcesData = reply.resources;
            }
          }
        }),
      ),
    };
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

.details-panel {
  border: solid;
  border-width: 1px;
  border-color: #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
}

.button {
  background-color: #5a7b7c;
}

.button-text {
  @apply font-normal;
  font-size: 11px;
  margin-top: 2px;
  margin-bottom: 2px;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
}
</style>
