<template>
  <div class="flex flex-row items-center w-full mt-2">
    <div class="w-40 px-2 text-sm leading-tight text-right text-white">
      name
    </div>
    <div
      class="flex flex-grow pl-2 mr-2 mr-10 text-sm leading-tight text-gray-400"
      v-if="editMode"
      @keyup.stop
      @keydown.stop
    >
      <input
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property"
        type="text"
        aria-label="name"
        placeholder="text"
        v-model="currentValue"
        @focus="onFocus"
        @blur="onBlur"
      />
    </div>
    <div
      v-else
      class="flex flex-grow pl-2 mr-2 text-sm leading-tight text-gray-400"
    >
      <template v-if="entity">
        {{ entity.name }}
      </template>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { Entity } from "@/api/sdf/model/entity";
import { updateEntity } from "@/observables";

interface Data {
  startValue: string;
  currentValue: string;
  updating: boolean;
}

export default Vue.extend({
  name: "TextField",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    editMode: {
      type: Boolean,
      required: true,
    },
    path: {
      type: Array as PropType<string[]>,
      required: true,
    },
    systemId: {
      type: String,
    },
    forName: {
      type: Boolean,
    },
  },
  data(): Data {
    return {
      startValue: "",
      currentValue: "",
      updating: false,
    };
  },
  methods: {
    onFocus() {
      this.setStartValueToCurrentValue();
      this.updating = true;
    },
    async onBlur() {
      this.updating = false;
      if (this.startValue != this.currentValue && this.forName) {
        this.entity.name = this.currentValue;
        updateEntity(this.entity).subscribe(reply => {
          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          }
        });
      }
    },
    setStartValueToCurrentValue() {
      this.startValue = _.cloneDeep(this.currentValue);
    },
    setCurrentValue(payload: string) {
      this.currentValue = payload;
    },
    updateOnPropChanges() {
      if (!this.updating && this.entity) {
        if (this.forName) {
          this.setCurrentValue(this.entity.name);
          this.setStartValueToCurrentValue();
        } else {
          const startValue: string = this.entity.getProperty({
            system: this.systemId,
            path: this.path,
          });
          this.setCurrentValue(_.cloneDeep(startValue));
          this.setStartValueToCurrentValue();
        }
      }
    },
  },
  watch: {
    entity: {
      deep: true,
      immediate: true,
      handler() {
        this.updateOnPropChanges();
      },
    },
  },
});
</script>
