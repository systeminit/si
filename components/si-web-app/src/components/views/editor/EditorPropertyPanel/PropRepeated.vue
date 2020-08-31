<template>
  <section :class="accordionClasses" v-if="fieldValue || editorMode == 'edit'">
    <div
      class="pl-2 text-sm text-white cursor-pointer section-header"
      @click="togglePath(entityProperty.path)"
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
    <div>
      <div class="flex flex-col text-gray-400">
        <div
          class="flex flex-row items-center pl-2 ml-16 mr-12"
          v-for="(repeatedEntry, index) in fieldValue"
          :key="index"
          v-show="showPath(entityProperty, index)"
        >
          <div class="w-full mb-2 border rounded-sm repeated-border">
            <div
              class="text-white"
              v-for="ep in propertiesList(entityProperty, index)"
              :key="ep.id"
            >
              <div v-if="!ep.hidden" class="flex flex-row">
                <div class="w-full" :style="propStyle(ep)">
                  <div class="py-1">
                    <div v-if="repeated(ep)">
                      <PropRepeated
                        :entityProperty="ep"
                        :backgroundColors="backgroundColors"
                        :collapsedPaths="collapsedPaths"
                        :isOpen="checkIsOpen(ep)"
                        @toggle-path="togglePath(ep.path)"
                      />
                    </div>

                    <div v-else-if="propKind(ep, 'object')">
                      <PropObject
                        :entityProperty="ep"
                        :isOpen="checkIsOpen(ep)"
                        @toggle-path="togglePath(ep.path)"
                      />
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

                    <div v-else-if="propKind(ep, 'select')">
                      <PropSelect :entityProperty="ep" />
                    </div>

                    <div v-else class="text-red-700">
                      {{ ep.name }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div v-show="showPath(entityProperty, index) && editorMode != 'view'">
            <button
              class="pl-1 text-gray-600 focus:outline-none"
              type="button"
              @click="removeFromList(index)"
            >
              <x-icon size="0.8x"></x-icon>
            </button>
          </div>
        </div>
        <div>
          <div
            class="flex justify-center text-gray-500 align-middle"
            v-if="editorMode == 'edit'"
          >
            <button
              class="w-4 pl-4 text-center focus:outline-none"
              type="button"
              @click="addToList"
            >
              <PlusSquareIcon size="1.25x" class=""></PlusSquareIcon>
            </button>
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
  XIcon,
} from "vue-feather-icons";
import _ from "lodash";

import { RootStore } from "@/store";
import PropText from "./PropText.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";
import PropSelect from "./PropSelect.vue";
import PropMixin from "./PropMixin";

import { RegistryProperty } from "@/api/sdf/model/node";

// This component only works with repeated objects! When we need it to work with
// repeated fields of other types, we're going to have to extend it. For now,
// no entity has a repeated but non-object field.
interface Data {
  collapsedPaths: (string | number)[][];
}

export default PropMixin.extend({
  name: "PropRepeated",
  props: {
    entityProperty: Object as () => RegistryProperty,
    backgroundColors: Array,
    collapsedPaths: Array,
    isOpen: Boolean,
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
    XIcon,
  },
  methods: {
    togglePath(path: (string | number)[]) {
      this.$emit("toggle-path", path);
    },
    propKind(prop: RegistryProperty, kindToCheck: string): boolean {
      return prop.kind == kindToCheck;
    },
    repeated(prop: RegistryProperty): boolean {
      return prop.repeated;
    },
    addToList(): void {
      if (!this.fieldValue) {
        this.fieldValue = [];
      }
      let current = _.cloneDeep(this.fieldValue);
      current.push({});
      this.fieldValue = current;
      //this.saveIfModified();
    },
    removeFromList(index: number): void {
      let current = _.cloneDeep(this.fieldValue);
      current.splice(index, 1);
      this.fieldValue = current;
      this.saveIfModified();
    },
    showPath(prop: RegistryProperty, index: number): boolean {
      let propPath = _.cloneDeep(prop.path);
      propPath.push(`${index}`);
      const collapsed = _.find(
        this.collapsedPaths,
        (path: (string | number)[]) => {
          if (propPath.length >= path.length) {
            if (!_.isEmpty(index)) {
              if (_.isEqual(propPath.slice(0, propPath.length - 1), path)) {
                // We always want to show the toggle path!
                return false;
              }
            } else {
              if (_.isEqual(propPath, path)) {
                // We always want to show the toggle path!
                return false;
              }
            }
            const sliceToCheck = propPath.slice(0, path.length);
            return _.isEqual(sliceToCheck, path);
          } else {
            return false;
          }
        },
      );
      if (collapsed) {
        return false;
      } else {
        return true;
      }
    },
    propStyle(entityProperty: RegistryProperty): string {
      let rgb: number[];
      if (
        entityProperty.name == "properties" &&
        entityProperty.path.length == 1
      ) {
        return "";
      } else {
        let maxDepth = this.backgroundColors.length;
        let epDepth = entityProperty.path.length;
        let depth;
        if (epDepth > maxDepth) {
          depth = maxDepth - 1;
        } else {
          depth = epDepth - 1;
        }
        // @ts-ignore
        rgb = this.backgroundColors[depth];
      }
      return `background-color: rgb(${rgb.join(",")});`;
    },
    checkIsOpen(prop: RegistryProperty): boolean {
      const collapsed = _.find(this.collapsedPaths, path => {
        if (_.isEqual(prop.path, path)) {
          return true;
        } else {
          return false;
        }
      });
      if (collapsed) {
        return false;
      } else {
        return true;
      }
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
  },
});
</script>

<style scoped>
.property-section-title-bg-color {
  background-color: #292c2d;
}
.repeated-border {
  border-color: #2b2e2f;
}
</style>
