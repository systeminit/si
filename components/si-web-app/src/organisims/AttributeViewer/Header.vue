<template>
  <section :class="accordionClasses" v-if="currentValue || editMode">
    <div
      class="flex w-full pt-1 pb-1 mt-2 text-sm text-white"
      @click="toggleAccordion"
      :style="propObjectStyle"
    >
      <div v-if="open" class="flex" :style="propObjectStyle">
        <ChevronDownIcon size="1.5x"></ChevronDownIcon>
        {{ editField.name }}
      </div>

      <div v-else-if="!open" class="flex" :style="propObjectStyle">
        <ChevronRightIcon size="1.5x"></ChevronRightIcon>
        {{ editField.name }}
      </div>
    </div>
  </section>
</template>

<style scoped>
.property-section-title-bg-color {
  background-color: #292c2d;
}

.section-content {
  @apply overflow-hidden transition duration-150 ease-in-out;
}

.is-closed .section-content {
  @apply overflow-hidden h-0;
}
</style>

<script lang="ts">
import Vue, { PropType } from "vue";

import { EditField } from "si-entity";
import { Entity } from "@/api/sdf/model/entity";
import _ from "lodash";

import { ChevronRightIcon, ChevronDownIcon } from "vue-feather-icons";

interface Data {
  currentValue: Record<string, any> | null;
}

export default Vue.extend({
  name: "Header",
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    editMode: {
      type: Boolean,
      required: true,
    },
    editField: {
      type: Object as PropType<EditField>,
      required: true,
    },
    systemId: {
      type: String,
    },
    backgroundColors: {
      type: Array as PropType<number[][]>,
      required: true,
    },
    outdentCount: {
      type: Number,
    },
    treeOpenState: {
      type: Object as PropType<{ [pathKey: string]: boolean }>,
      required: true,
    },
  },
  data(): Data {
    return {
      currentValue: null,
    };
  },
  computed: {
    open(): boolean {
      const key = this.editField.path.join("::");
      const openState = this.treeOpenState[key];
      return openState;
    },
    paddingLeft(): number {
      const indentFactorPx = 10;
      let indentCount = this.editField.path.length;
      if (!_.isUndefined(this.outdentCount)) {
        indentCount -= this.outdentCount;
      }

      return indentCount * indentFactorPx;
    },
    propObjectStyle(): string {
      const rgb = this.backgroundColors[this.editField.path.length - 1].join(
        ",",
      );
      let style = `background-color: rgb(${rgb});`;
      style = `${style} padding-left: ${this.paddingLeft}px;`;
      return style;
    },
    accordionClasses(): Record<string, boolean> {
      return {
        "is-closed": !this.open,
      };
    },
  },
  methods: {
    toggleAccordion(): void {
      const pathKey = this.editField.path.join("::");
      this.$emit("toggle-path", pathKey);
    },
    updateOnPropChanges() {
      if (this.entity) {
        const startValue: Record<string, any> = this.entity.getProperty({
          system: this.systemId,
          path: this.editField.path,
        });
        this.setCurrentValue(_.cloneDeep(startValue));
      }
    },
    setCurrentValue(payload: Record<string, any>) {
      this.currentValue = payload;
    },
  },
  created(): void {
    const key = this.editField.path.join("::");
    if (!this.treeOpenState.hasOwnProperty(key)) {
      // TDOD(fnichol): is it false always by default??
      this.$emit("set-tree-open-state", { key, value: false });
    }
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
