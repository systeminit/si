<template>
  <section :class="accordionClasses" v-if="fieldValue || editorMode == 'edit'">
    <div
      v-if="isProperties"
      class="mt-1 mb-1 ml-6 text-base text-white align-middle"
    >
      Properties
    </div>
    <div
      v-else
      class="pl-2 text-sm text-white cursor-pointer section-header"
      @click="toggleAccordion"
    >
      <div v-if="isOpen" class="flex" :style="propObjectStyle">
        <chevron-down-icon size="1.5x"></chevron-down-icon>
        {{ entityProperty.name }}
      </div>

      <div v-else-if="!isOpen" class="flex" :style="propObjectStyle">
        <chevron-right-icon size="1.5x"></chevron-right-icon>
        {{ entityProperty.name }}
      </div>
    </div>
  </section>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";

import { RegistryProperty } from "@/store/modules/node";

interface PropObjectData {
  isOpen: boolean;
}

export default Vue.extend({
  name: "PropObject",
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
  },
  props: {
    entityProperty: Object as () => RegistryProperty,
    isOpen: Boolean,
  },
  methods: {
    toggleAccordion(): void {
      this.$emit("toggle-path", this.entityProperty.path);
    },
  },
  computed: {
    isProperties(): boolean {
      return (
        this.entityProperty.name == "properties" &&
        this.entityProperty.path.length == 1
      );
    },
    ...mapState({
      editorMode: (state: any) => state.editor.mode,
    }),
    propObjectStyle(): string {
      if (this.entityProperty.path.length == 1) {
        return "";
      }
      let results = `margin-left: ${this.entityProperty.path.length * 10}px`;
      return results;
    },
    accordionClasses(): { "is-closed": boolean } {
      return {
        "is-closed": !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    },
    fieldValue(): any {
      let value = this.$store.getters["node/getFieldValue"](
        this.entityProperty.path,
      );
      return value;
    },
  },
});
</script>

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
