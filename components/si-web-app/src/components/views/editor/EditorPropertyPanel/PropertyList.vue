<template>
  <div>
    <div v-if="selectedNode">
      <div
        class="flex pt-2 pb-2 pl-6 text-base text-white align-middle property-section-bg-color"
      >
        <div class="self-center w-3/4 text-lg ">
          {{ typeName }}
        </div>
        <div class="flex justify-end w-2/4">
          <div
            v-if="diff.count"
            class="self-center mr-4 text-xs text-right align-middle"
          >
            <EditIcon size="1x" class="inline mr-1 gold-bars-icon" />
            Edit Count: {{ diff.count }}
          </div>
        </div>
      </div>
      <div class="text-red-700" v-if="selectedNode.deleted">
        Will be deleted!
      </div>

      <div class="flex items-center mt-2">
        <div class="w-40 px-2 text-sm leading-tight text-right text-white">
          name
        </div>
        <div
          v-if="editorMode == 'view'"
          v-bind:class="textClasses"
          class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
        >
          {{ nodeObjectName }}
        </div>
        <div
          class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
          v-else-if="editorMode == 'edit'"
        >
          <input
            class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey input-border-grey si-property"
            type="text"
            v-bind:class="inputClasses"
            aria-label="name"
            v-model="nodeObjectName"
            @blur="updateObjectName"
            placeholder="text"
          />
        </div>
      </div>

      <div
        class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
      >
        Properties
      </div>

      <div v-for="entityProperty in propertiesList" :key="entityProperty.id">
        <div v-if="!entityProperty.hidden" class="flex flex-row">
          <div
            class="w-full"
            :style="propStyle(entityProperty)"
            v-show="showPath(entityProperty)"
          >
            <div
              v-if="
                repeated(entityProperty) && !propKind(entityProperty, 'select')
              "
            >
              <PropRepeated
                :entityProperty="entityProperty"
                :isOpen="isOpen(entityProperty)"
                :backgroundColors="backgroundColors"
                :collapsedPaths="collapsedPaths"
                class="py-2"
                @toggle-path="togglePath($event)"
              />
            </div>

            <div v-else-if="propKind(entityProperty, 'object')">
              <PropObject
                :entityProperty="entityProperty"
                :isOpen="isOpen(entityProperty)"
                class="py-2"
                @toggle-path="togglePath($event)"
              />
            </div>

            <div v-else-if="propKind(entityProperty, 'text')">
              <PropText :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else-if="propKind(entityProperty, 'code')">
              <!-- for now, do nothing! -->
            </div>

            <div v-else-if="propKind(entityProperty, 'number')">
              <PropNumber :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else-if="propKind(entityProperty, 'enum')">
              <PropEnum :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else-if="propKind(entityProperty, 'bool')">
              <PropBool :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else-if="propKind(entityProperty, 'map')">
              <PropMap :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else-if="propKind(entityProperty, 'select')">
              <PropSelect :entityProperty="entityProperty" class="py-1" />
            </div>

            <div v-else class="py-1 text-red-700">
              {{ entityProperty.name }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { mapState, mapGetters } from "vuex";

import PropText from "./PropText.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";
import PropRepeated from "./PropRepeated.vue";
import PropBool from "./PropBool.vue";
import PropSelect from "./PropSelect.vue";
import { Node, RegistryProperty } from "@/api/sdf/model/node";
//import { RegistryProperty } from "../../../../store/modules/node";

import { capitalCase } from "change-case";
import { EditIcon } from "vue-feather-icons";
import _ from "lodash";

interface Data {
  collapsedPaths: (string | number)[][];
  nodeObjectName: string;
}

export default Vue.extend({
  name: "PropertyList",
  components: {
    PropText,
    PropObject,
    PropNumber,
    PropEnum,
    PropMap,
    PropRepeated,
    PropBool,
    PropSelect,
    EditIcon,
  },
  props: {
    selectedNode: {
      type: Object as PropType<Node | undefined>,
    },
  },
  data(): Data {
    return {
      collapsedPaths: [],
      nodeObjectName: "",
    };
  },
  methods: {
    async updateObjectName() {
      await this.$store.dispatch("editor/entityNameSet", {
        value: this.nodeObjectName,
      });
    },
    togglePath(path: (string | number)[]) {
      if (
        _.find(this.collapsedPaths, item => {
          return _.isEqual(item, path);
        })
      ) {
        const newPaths = [];
        for (const item of this.collapsedPaths) {
          if (!_.isEqual(item, path)) {
            newPaths.push(item);
          }
        }
        this.collapsedPaths = newPaths;
      } else {
        this.collapsedPaths.push(path);
      }
    },
    isOpen(prop: RegistryProperty): boolean {
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
    showPath(prop: RegistryProperty): boolean {
      const collapsed = _.find(this.collapsedPaths, path => {
        if (prop.path.length >= path.length) {
          if (_.isEqual(prop.path, path)) {
            // We always want to show the toggle path!
            return false;
          }
          const sliceToCheck = prop.path.slice(0, path.length);
          return _.isEqual(sliceToCheck, path);
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
    propKind(prop: RegistryProperty, kindToCheck: string): boolean {
      return prop.kind == kindToCheck;
    },
    repeated(prop: RegistryProperty): boolean {
      return prop.repeated;
    },
    // Returns a single rgb color interpolation between given rgb color
    // based on the factor given; via https://codepen.io/njmcode/pen/axoyD?editors=0010
    interpolateColor(
      color1: number[],
      color2: number[],
      factor: number,
    ): number[] {
      if (arguments.length < 3) {
        factor = 0.5;
      }
      let result: number[] = color1.slice();
      for (var i = 0; i < 3; i++) {
        result[i] = Math.round(result[i] + factor * (color2[i] - color1[i]));
      }
      return result;
    },
    // My function to interpolate between two colors completely, returning an array
    interpolateColors(
      color1input: string,
      color2input: string,
      steps: number,
    ): number[][] {
      var stepFactor = 1 / (steps - 1),
        interpolatedColorArray = [];

      const color1: undefined | number[] = color1input
        .match(/\d+/g)
        ?.map(Number);
      if (color1 === undefined) {
        throw new Error(`Cannot parse color input for: ${color1input}`);
      }
      const color2: undefined | number[] = color2input
        .match(/\d+/g)
        ?.map(Number);
      if (color2 === undefined) {
        throw new Error(`Cannot parse color input for: ${color2input}`);
      }

      for (var i = 0; i < steps; i++) {
        interpolatedColorArray.push(
          this.interpolateColor(color1, color2, stepFactor * i),
        );
      }

      return interpolatedColorArray;
    },
    propStyle(entityProperty: RegistryProperty): string {
      let rgb: number[];
      if (
        entityProperty.name == "properties" &&
        entityProperty.path.length == 1
      ) {
        return "";
      } else {
        rgb = this.backgroundColors[entityProperty.path.length - 1];
      }
      return `background-color: rgb(${rgb.join(",")});`;
    },
  },
  computed: {
    typeName(): string {
      return capitalCase(this.selectedNode?.objectType || "unknown");
    },
    ...mapState({
      propertiesList: (state: any): RegistryProperty[] =>
        state.editor.propertyList,
      editorMode: (state: any): any => state.editor.mode,
      editObject: (state: any): any => state.editor.editObject,
      diff: (state: any): any => state.editor.diff,
    }),
    backgroundColors(): number[][] {
      let longestProp = 0;
      for (const property of this.propertiesList) {
        if (property.path) {
          if (property.path.length > longestProp) {
            longestProp = property.path.length;
          }
        }
      }
      longestProp = longestProp;
      const colors = this.interpolateColors(
        "rgb(50, 50, 50)",
        "rgb(25, 25, 25)",
        longestProp,
      );
      return colors;
    },
    textClasses(): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      if (this.hasBeenEdited) {
        results["input-border-gold"] = true;
        results["border"] = true;
      } else {
        results["input-border-grey"] = true;
      }
      return results;
    },
    inputClasses(): Record<string, boolean> {
      let results: Record<string, boolean> = {};
      results["si-property"] = true;
      if (this.hasBeenEdited) {
        results["input-border-gold"] = true;
        results["input-bg-color-grey"] = true;
      } else {
        results["input-border-grey"] = true;
        results["input-bg-color-grey"] = true;
      }
      return results;
    },
    hasBeenEdited(): boolean {
      let result = _.find(this.diff.entries, diffEntry => {
        return _.isEqual(diffEntry.path, ["name"]);
      });
      if (result) {
        return true;
      } else {
        return false;
      }
    },
  },
  watch: {
    selectedNode(value: any): void {
      this.collapsedPaths = [];
    },
    editObject(value: any): void {
      if (this.editObject?.name) {
        this.nodeObjectName = _.cloneDeep(this.editObject.name);
      }
    },
  },
  async created() {
    await this.$store.dispatch("editor/loadEditObject");
  },
});
</script>

<style scoped>
.gold-bars-icon {
  color: #ce7f3e;
}

.property-section-bg-color {
  background-color: #292c2d;
}
</style>
