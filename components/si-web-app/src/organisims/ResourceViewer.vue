<template>
  <div class="flex flex-col w-full" v-if="entity">
    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-white property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} resource
      </div>
    </div>

    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base bg-black"
    >
      <div class="flex">
        <button
          class="flex items-center pl-1 focus:outline-none"
          v-show="!editMode"
          @click="runSync()"
        >
          <RefreshCcwIcon size="1x" class="text-sm button" />
        </button>
      </div>
    </div>

    <div class="flex w-full h-full pt-2 overflow-scroll">
      <div class="flex w-full h-full text-xs" v-if="resource">
        <ResourceVisualization :resource="resource" />
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
import ResourceVisualization from "@/organisims/ResourceViewer/ResourceVisualization.vue";
import { RefreshCcwIcon } from "vue-feather-icons";

interface IData {
  isSynchronizing: boolean;
}

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
  data(): IData {
    return {
      isSynchronizing: false,
    };
  },
  components: {
    // VueJsonPretty,
    RefreshCcwIcon,
    ResourceVisualization,
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
        this.isSynchronizing = true;
        let reply = await ResourceDal.syncResource({
          entityId: this.entity.id,
          // @ts-ignore
          systemId: this.system.id,
          // @ts-ignore
          workspaceId: this.workspace.id,
        });
        this.isSynchronizing = false;
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
$button-brightness: 1.1;

.property-section-bg-color {
  background-color: #292c2d;
}

.header-background {
  background-color: #1f2122;
}

.button {
  color: #05b5bc;
  // color: #98E9F5;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
  @apply animate-ping;
}
</style>
