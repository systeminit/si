<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 pt-2 pb-2 pl-6 pr-6 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }}
      </div>
      <div
        class="flex flex-row items-center items-end justify-end flex-grow h-full text-xs text-center"
      >
        <div v-if="diff">
          <EditIcon size="1x" class="mr-1 gold-bars-icon" />
        </div>
        <div v-if="diff" class="text-center">Edit Count: {{ diff.length }}</div>
        <div class="ml-2">
          <Tooltip class="inline" alignRight :offset="0.5" sticky>
            <InfoIcon size="1x" class="inline mr-1" />
            <template v-slot:tooltip>
              <div class="flex flex-col text-sm text-gray-400">
                <div class="pl-2">
                  {{ entity.nodeId }}
                </div>
                <div class="pl-2">
                  {{ entity.id }}
                </div>
              </div>
            </template>
          </Tooltip>
        </div>
      </div>
    </div>

    <div class="flex flex-col w-full overflow-auto overscroll-none">
      <NameField
        :entity="entity"
        :editMode="editMode"
        :path="[]"
        :systemId="systemId"
        :forName="true"
      />
      <div
        class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
        v-if="editFields && editFields.length"
      >
        Properties
      </div>
      <template v-if="editFields">
        <div v-for="editField in editFields" :key="editField.path.join('.')">
          <TextField
            :entity="entity"
            :editMode="editMode"
            :editField="editField"
            :systemId="systemId"
            v-if="editField.widgetName == 'text'"
          />
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.gold-bars-icon {
  color: #ce7f3e;
}

.property-section-bg-color {
  background-color: #292c2d;
}
</style>

<script lang="ts">
import Vue, { PropType } from "vue";

import Tooltip from "@/atoms/Tooltip.vue";
import { EditIcon, InfoIcon } from "vue-feather-icons";
import TextField from "@/organisims/AttributeViewer/TextField.vue";
import NameField from "@/organisims/AttributeViewer/NameField.vue";
import { Entity } from "@/api/sdf/model/entity";
import { editMode$, system$ } from "@/observables";
import { Diff } from "@/api/sdf/model/diff";

export default Vue.extend({
  name: "AttributeViewer",
  components: {
    Tooltip,
    InfoIcon,
    EditIcon,
    TextField,
    NameField,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    diff: {
      type: Array as PropType<Diff>,
      required: true,
    },
  },
  subscriptions() {
    return {
      editMode: editMode$,
      system: system$,
    };
  },
  computed: {
    editFields(): ReturnType<Entity["editFields"]> | undefined {
      if (this.entity) {
        return this.entity.editFields();
      } else {
        return undefined;
      }
    },
    systemId(): string {
      // @ts-ignore
      if (this.system) {
        // @ts-ignore
        return this.system.id;
      } else {
        return "baseline";
      }
    },
  },
});
</script>
