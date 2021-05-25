<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base text-white property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} resource
      </div>
    </div>

    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base text-white bg-black"
    >
      <div class="flex">
        <button
          class="pl-1 focus:outline-none disabled:opacity-30"
          :disabled="editMode"
          @click="runSync()"
        >
          <ZapIcon size="1.1x" />
        </button>
      </div>
    </div>

    <div class="flex w-full pt-2 overflow-auto">
      <div class="flex flex-col w-full" v-if="resource">
        <VueJsonPretty :data="resource" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Entity } from "@/api/sdf/model/entity";
import { Resource } from "@/api/sdf/model/resource";
import { ResourceDal } from "@/api/sdf/dal/resourceDal";
import { editMode$, system$, workspace$ } from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { ZapIcon } from "vue-feather-icons";
import VueJsonPretty from "vue-json-pretty";

export default Vue.extend({
  name: "ResourceViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    VueJsonPretty,
    ZapIcon,
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      system: system$,
      workspace: workspace$,
    };
  },
  methods: {
    async runSync(): Promise<void> {
      // @ts-ignore
      if (this.system && this.workspace) {
        let reply = await ResourceDal.syncResource({
          entityId: this.entity.id,
          // @ts-ignore
          systemId: this.system.id,
          // @ts-ignore
          workspaceId: this.workspace.id,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        }
      }
    },
  },
});
</script>
