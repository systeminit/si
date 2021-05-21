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
        class="flex-grow pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property"
        :class="borderColor"
        type="text"
        aria-label="name"
        placeholder="text"
        v-model="currentValue"
        @keyup.enter="onEnterKey($event)"
        @focus="onFocus"
        @blur="onBlur"
      />
    </div>
    <div
      v-else
      class="flex flex-grow pl-2 mr-2 text-sm leading-tight text-gray-400"
    >
      <template v-if="entity">
        <span :class="textColor"> {{ entity.name }} </span>
      </template>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { Entity } from "@/api/sdf/model/entity";
import { updateEntity, nameAttributeChanged$ } from "@/observables";
import { Diff, hasDiff } from "@/api/sdf/model/diff";

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
    diff: {
      type: Array as PropType<Diff>,
    },
  },
  data(): Data {
    return {
      startValue: "",
      currentValue: "",
      updating: false,
    };
  },
  computed: {
    borderColor(): Record<string, boolean> {
      const gold = hasDiff(this.diff, ["name"]);
      if (gold) {
        return {
          "input-border-gold": true,
        };
      } else {
        return {
          "input-border-grey": true,
        };
      }
    },
    textColor(): Record<string, boolean> {
      const gold = hasDiff(this.diff, ["name"]);
      if (gold) {
        return {
          "text-gold": true,
        };
      } else {
        return {
          "text-gold": false,
        };
      }
    },
  },
  methods: {
    onEnterKey(event: KeyboardEvent) {
      if (event.target) {
        // @ts-ignore
        event.target.blur();
      }
    },
    onFocus() {
      this.setStartValueToCurrentValue();
      this.updating = true;
    },
    async onBlur() {
      this.updating = false;
      if (this.startValue != this.currentValue && this.forName) {
        this.entity.name = this.currentValue;
        this.entity.computeProperties();
        // Name must be committed to the api before notifying rest of client,
        // hence awaited here
        const reply = await updateEntity(this.entity).toPromise();
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
          return;
        }
        // Now rest of client can be notified which may cause refetching of
        // data including schematics, hence comment above about api being
        // consistent *first*
        nameAttributeChanged$.next({
          nodeId: this.entity.nodeId,
          entityId: this.entity.id,
          entityType: this.entity.entityType,
          oldValue: this.startValue,
          newValue: this.currentValue,
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
