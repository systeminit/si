<template>
  <div class="w-full h-full">
    <SummaryCard>
      <template v-slot:title>Services</template>

      <template v-slot:content>
        <div class="flex flex-col content-start w-full h-full">
          <div class="flex flex-row flex-wrap w-full h-full mx-1">
            <div
              class="mr-2"
              v-for="resource in servicesData"
              :key="resource.id"
            >
              <ServiceVisualization :resource="resource" />
            </div>
          </div>

          <div class="flex justify-end mt-2" v-show="showButton">
            <div
              class="flex text-xs text-center align-middle button"
              @click="deploy()"
              v-show="deployAble"
            >
              <div class="mx-1 button-text">Deploy</div>
            </div>
          </div>
        </div>
      </template>
    </SummaryCard>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import ServiceVisualization from "@/molecules/ServicesSummary/ServiceVisualization.vue";
import SummaryCard from "@/atoms/SummaryCard.vue";
import {
  editMode$,
  system$,
  workspace$,
  refreshActivitySummary$,
  resources$,
  refreshResourceSummary$,
  changeSet$,
} from "@/observables";
import {
  ApplicationDal,
  IResourceSummaryReplySuccess,
  ResourceSummaryKind,
} from "@/api/sdf/dal/applicationDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { pluck, tap, switchMap } from "rxjs/operators";
import { combineLatest, from } from "rxjs";

interface IData {
  servicesData: IResourceSummaryReplySuccess["resources"];
}

export default Vue.extend({
  name: "ServicesSummary",
  components: {
    ServiceVisualization,
    SummaryCard,
  },
  props: {
    showButton: {
      type: Boolean,
      default: true,
    },
    applicationId: {
      type: String,
    },
  },
  data(): IData {
    return {
      servicesData: [],
    };
  },
  subscriptions: function(this: any): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));
    return {
      deployAble: combineLatest(editMode$, changeSet$).pipe(
        switchMap(([editMode, changeSet]) => {
          if (!editMode && !changeSet) {
            return from([true]);
          } else {
            return from([false]);
          }
        }),
      ),
      editMode: editMode$,
      system: system$,
      workspace: workspace$,
      resources: resources$.pipe(
        tap(r => {
          let isUpdated = false;
          // @ts-ignore
          if (this.servicesData) {
            // @ts-ignore
            for (let x = 0; x < this.servicesData.length; x++) {
              // @ts-ignore
              if (r.id == this.servicesData[x].id) {
                isUpdated = true;
                Vue.set(this.servicesData, x, r);
              }
            }
            if (!isUpdated) {
              if (r.entityType == "service") {
                this.servicesData.push(r);
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
              // @ts-ignore
              this.servicesData = reply.resources;
            }
          }
        }),
      ),
    };
  },
  methods: {
    async deploy(): Promise<void> {
      // @ts-ignore
      if (this.applicationId && this.system && this.workspace) {
        const reply = await ApplicationDal.deployServices({
          // @ts-ignore
          workspaceId: this.workspace.id,
          // @ts-ignore
          systemId: this.system.id,
          // @ts-ignore
          applicationId: this.applicationId,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        } else {
          refreshActivitySummary$.next(true);
        }
      }
    },
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.1;

.button {
  background-color: #5a8f91;
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

.button:disabled {
  background-color: red;
  color: red;
}
</style>
