<template>
  <section :class="accordionClasses" v-if="fieldValue || editorMode == 'edit'">
    <div
      class="section-header cursor-pointer pl-2 text-sm text-white property-section-title-bg-color"
      @click="toggleAccordion"
    >
      <div v-if="isOpen" :class="`flex ml-${entityProperty.path.length}`">
        <chevron-down-icon size="1.5x"></chevron-down-icon>
        {{ entityProperty.name }}
      </div>

      <div v-else-if="!isOpen" :class="`flex ml-${entityProperty.path.length}`">
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

import { EntityProperty } from "@/store/modules/entity";

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
    entityProperty: Object as () => EntityProperty,
  },
  data(): PropObjectData {
    return {
      isOpen: true,
    };
  },
  methods: {
    toggleAccordion(): void {
      this.isOpen = !this.isOpen;
    },
  },
  computed: {
    ...mapState({
      editorMode: (state: any) => state.editor.mode,
    }),
    accordionClasses(): { "is-closed": boolean } {
      return {
        "is-closed": !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    },
    fieldValue: {
      get(): Object {
        let value = this.$store.getters["editor/getEditValue"](
          this.entityProperty.path,
        );
        return value;
      },
      async set(value: any) {
        await this.$store.dispatch("editor/setEditValue", {
          path: this.entityProperty.path,
          value,
        });
      },
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
