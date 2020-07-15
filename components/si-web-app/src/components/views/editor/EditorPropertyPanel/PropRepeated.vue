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

    <div class="flex text-gray-500" v-if="editorMode == 'edit'">
      <button
        class="text-center w-4 pl-4 focus:outline-none"
        type="button"
        @click="addToList"
      >
        <PlusSquareIcon size="1.25x" class=""></PlusSquareIcon>
      </button>
    </div>

    <div v-for="(repeatedEntry, index) in fieldValue" :key="index">
      <div class="text-white">{{ entityProperty.name }} {{ index }}</div>
      <div
        class="text-white"
        v-for="ep in propertiesList(entityProperty, index)"
        :key="ep.path.join('-')"
      >
        <div v-if="!ep.hidden" class="flex flex-row">
          <div class="w-full">
            <div class="py-1">
              <div v-if="repeated(ep)">
                <PropRepeated :entityProperty="ep" />
              </div>

              <div v-else-if="propKind(ep, 'object')">
                <PropObject :entityProperty="ep" />
              </div>

              <div v-else-if="propKind(ep, 'text')">
                <PropText :entityProperty="ep" />
              </div>

              <div v-else-if="propKind(ep, 'code')">
                <!-- for now, do nothing! -->
              </div>

              <div v-else-if="propKind(ep, 'number')">
                <PropNumber :entityProperty="ep" />
              </div>

              <div v-else-if="propKind(ep, 'enum')">
                <PropEnum :entityProperty="ep" />
              </div>

              <div v-else-if="propKind(ep, 'map')">
                <PropMap :entityProperty="ep" />
              </div>

              <div v-else class="text-red-700">
                {{ ep.name }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import {
  ChevronDownIcon,
  ChevronRightIcon,
  PlusSquareIcon,
} from "vue-feather-icons";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty, debouncedSetFieldValue } from "@/store/modules/node";
import PropText from "./PropText.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";

// This component only works with repeated objects! When we need it to work with
// repeated fields of other types, we're going to have to extend it. For now,
// no entity has a repeated but non-object field.

export default Vue.extend({
  name: "PropRepeated",
  props: {
    entityProperty: Object as () => RegistryProperty,
  },
  components: {
    PropText,
    PropObject,
    PropNumber,
    PropEnum,
    PropMap,
    ChevronRightIcon,
    ChevronDownIcon,
    PlusSquareIcon,
  },
  data(): { isOpen: boolean } {
    return {
      isOpen: true,
    };
  },
  methods: {
    toggleAccordion(): void {
      this.isOpen = !this.isOpen;
    },
    propKind(prop: RegistryProperty, kindToCheck: string): boolean {
      return prop.kind == kindToCheck;
    },
    repeated(prop: RegistryProperty): boolean {
      return prop.repeated;
    },
    addToList(): void {
      let current = _.cloneDeep(this.fieldValue);
      current.push({});
      this.fieldValue = current;
    },
    removeFromList(index: number): void {
      let current = _.cloneDeep(this.fieldValue);
      current.splice(index, 1);
      this.fieldValue = current;
    },
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
    ...mapGetters({
      propertiesList: "node/propertiesListRepeated",
    }),
    accordionClasses(): { "is-closed": boolean } {
      return {
        "is-closed": !this.isOpen,
        // 'is-primary': this.isOpen,
        // 'is-dark': !this.isOpen
      };
    },
    fieldValue: {
      get(): any[] {
        let objectValues = this.$store.getters["node/getFieldValue"](
          this.entityProperty.path,
        );
        if (objectValues != undefined) {
          return _.cloneDeep(objectValues);
        } else {
          return [];
        }
      },
      async set(value: any) {
        await debouncedSetFieldValue({
          store: this.$store,
          path: this.entityProperty.path,
          value: value,
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
