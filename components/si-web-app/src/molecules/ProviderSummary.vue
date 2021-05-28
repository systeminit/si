<template>
  <SummaryCard>
    <template v-slot:title>Providers</template>

    <template v-slot:content>
      <div class="flex flex-col w-full h-full">
        <div class="flex flex-row flex-wrap w-full h-full mx-1">
          <div class="mr-2" v-for="resource in providerData" :key="resource.id">
            <ResourceVisualization :resource="resource" />
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
import { pluck, tap } from "rxjs/operators";
import {
  workspace$,
  system$,
  resources$,
  refreshResourceSummary$,
} from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  providerData: IResourceSummaryReplySuccess["resources"];
}

export default Vue.extend({
  name: "ProviderSummary",
  components: {
    ResourceVisualization,
    SummaryCard,
  },
  props: {
    applicationId: {
      type: String,
    },
  },
  data(): IData {
    return {
      providerData: [],
    };
  },
  subscriptions(): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));

    return {
      resources: resources$.pipe(
        tap(r => {
          let isUpdated = false;
          // @ts-ignore
          if (this.providerData) {
            // @ts-ignore
            for (let x = 0; x < this.providerData.length; x++) {
              // @ts-ignore
              if (r.id == this.providerData[x].id) {
                isUpdated = true;
                // @ts-ignore
                Vue.set(this.providerData, x, r);
              }
            }
            if (!isUpdated) {
              if (r.entityType == "cloudProvider") {
                // @ts-ignore
                this.providerData.push(r);
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
              applicationId,
              workspaceId: workspace.id,
              systemId: system.id,
              kind: ResourceSummaryKind.Providers,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              // @ts-ignore
              this.providerData = reply.resources;
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
