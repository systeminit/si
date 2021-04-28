<template>
  <section :class="accordionClasses" v-if="currentValue || editMode">
    <div
      class="text-sm text-white cursor-pointer pt-1 pb-1 mt-2"
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
  open: boolean;
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
  },
  data(): Data {
    return {
      open: true,
      currentValue: null,
    };
  },
  computed: {
    propObjectStyle(): string {
      const rgb = this.backgroundColors[this.editField.path.length - 1].join(
        ",",
      );
      let style = `background-color: rgb(${rgb});`;
      style = `${style} padding-left: ${this.editField.path.length * 10}px;`;
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
      this.open = !this.open;
      this.$emit("toggle-path", this.editField.path);
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
