<template>
  <section :class="accordionClasses" v-if="fieldValue || editMode">
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
        {{ registryProperty.name }}
      </div>

      <div v-else-if="!isOpen" class="flex" :style="propObjectStyle">
        <chevron-right-icon size="1.5x"></chevron-right-icon>
        {{ registryProperty.name }}
      </div>
    </div>
  </section>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { mapState } from "vuex";
import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";

import { RegistryProperty } from "@/api/registryProperty";
import _ from "lodash";
import { Entity } from "@/api/sdf/model/entity";

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
    registryProperty: Object as PropType<RegistryProperty>,
    editObject: Object as PropType<Entity>,
    isOpen: Boolean,
  },
  methods: {
    toggleAccordion(): void {
      this.$emit("toggle-path", this.registryProperty.path);
    },
  },
  computed: {
    isProperties(): boolean {
      return (
        this.registryProperty.name == "properties" &&
        this.registryProperty.path.length == 1
      );
    },
    editMode(): boolean {
      return this.$store.getters["editor/inEditable"];
    },
    propObjectStyle(): string {
      if (this.registryProperty.path.length == 1) {
        return "";
      }
      let results = `margin-left: ${this.registryProperty.path.length * 10}px`;
      return results;
    },
    accordionClasses(): { "is-closed": boolean } {
      return {
        "is-closed": !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    },
    fieldValue(): boolean {
      let fieldValue: any = _.get(
        this.editObject.properties["__baseline"],
        this.registryProperty.path,
      );
      return !!fieldValue;
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
