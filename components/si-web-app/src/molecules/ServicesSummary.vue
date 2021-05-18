<template>
  <div class="w-full h-full">
    <SummaryCard>
      <template v-slot:title>Services</template>

      <template v-slot:content>
        <div class="flex flex-col w-full h-full">
          <div class="flex flex-row flex-wrap w-full h-full mx-1">
            <div
              class="mr-2"
              v-for="(service, index) in servicesData"
              :key="index"
            >
              <ServiceVisualization :data="service" />
            </div>
          </div>

          <div class="flex justify-end mt-2" v-show="showButton">
            <div class="flex items-center justify-center button">
              <!-- <upload-icon size="0.75x" class="mx-1 my-1 align-middle" /> -->
              <button
                class="mx-1 align-middle button-text disabled:opacity-30"
                :disabled="editMode"
                @click="deploy()"
              >
                Deploy
              </button>
            </div>
          </div>
        </div>
      </template>
    </SummaryCard>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import { servicesData, Service } from "@/api/visualization/servicesData";
import ServiceVisualization from "@/molecules/ServicesSummary/ServiceVisualization.vue";
import SummaryCard from "@/atoms/SummaryCard.vue";
import { editMode$, system$, workspace$, applicationId$ } from "@/observables";
import { ApplicationDal } from "@/api/sdf/dal/applicationDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  servicesData: Service[];
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
  },
  data(): IData {
    return {
      servicesData: servicesData,
    };
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      system: system$,
      workspace: workspace$,
      applicationId: applicationId$,
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
        }
      }
    },
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

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
