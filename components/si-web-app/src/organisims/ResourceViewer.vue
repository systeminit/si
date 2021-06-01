<template>
  <div class="flex flex-col w-full" v-if="entity">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} resource
      </div>

      <div class="flex pl-1">
        <button
          class="flex items-center focus:outline-none button"
          ref="sync"
          v-if="!editMode && resource"
          @click="runSync()"
        >
          <RefreshCwIcon size="1x" class="text-sm" :class="healthColor" />
        </button>
        <BoxIcon size="1x" class="text-base" :class="healthColor" v-else />
      </div>
    </div>

    <div class="flex flex-row">
      <div class="w-full h-full pt-2">
        <div class="flex flex-col w-full h-full" v-if="resource">
          <ResourceVisualization :resource="resource" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Entity } from "@/api/sdf/model/entity";
import { Resource, ResourceHealth } from "@/api/sdf/model/resource";
import { ResourceDal } from "@/api/sdf/dal/resourceDal";
import { editMode$, system$, workspace$ } from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import ResourceVisualization from "@/organisims/ResourceViewer/ResourceVisualization.vue";
import { RefreshCwIcon, BoxIcon } from "vue-feather-icons";

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
    RefreshCwIcon,
    ResourceVisualization,
    BoxIcon,
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      system: system$,
      workspace: workspace$,
    };
  },
  computed: {
    healthColor(): Record<string, any> {
      let style: Record<string, any> = {};

      if (this.resource) {
        if (this.resource.health == ResourceHealth.Ok) {
          style["health-ok"] = true;
        } else if (this.resource.health == ResourceHealth.Warning) {
          style["health-warning"] = true;
        } else if (this.resource.health == ResourceHealth.Error) {
          style["health-error"] = true;
        } else if (this.resource.health == ResourceHealth.Unknown) {
          style["health-unknown"] = true;
        } else {
          style["health-unknown"] = true;
        }
      }
      return style;
    },
  },
  methods: {
    async runSync(): Promise<void> {
      this.animateSyncButton();
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
    animateSyncButton() {
      const button = this.$refs.sync as HTMLElement;
      if (button) {
        button.animate(
          [{ transform: "rotate(0deg)" }, { transform: "rotate(720deg)" }],
          {
            duration: 2500,
            easing: "linear",
          },
        );
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

.sync-button-invert {
  transform: scaleX(-1);
}

.health-ok {
  color: #86f0ad;
}

.health-warning {
  color: #f0d286;
}

.health-error {
  color: #f08686;
}

.health-unknown {
  color: #bbbbbb;
}
</style>
