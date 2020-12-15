<template>
  <div class="relative w-full h-full">
    <div class="w-full h-full" v-if="selectedNode">
      <div
        class="flex pt-2 pb-2 pl-6 pr-6 text-base text-white align-middle property-section-bg-color"
      >
        <div class="self-center w-3/4 text-lg">
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

          <Tooltip alightRight :offset="tooltipOffset" sticky>
            <info-icon size="1x" class="inline mr-1" />
            <template v-slot:tooltip>
              <div class="flex flex-col text-sm text-gray-400">
                <div class="flex">
                  <div v-if="selectedNode" class="pl-2">
                    {{ selectedNode.id }}
                  </div>
                </div>
                <div class="flex">
                  <div v-if="editObject" class="pl-2">
                    {{ editObject.id }}
                  </div>
                </div>
              </div>
            </template>
          </Tooltip>
        </div>
      </div>
      <div class="text-red-700" v-if="selectedNode.deleted">
        Will be deleted!
      </div>

      <div class="flex flex-col w-full h-full overflow-auto">
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
              data-cy="editor-property-viewer-node-name-field"
              placeholder="text"
            />
          </div>
        </div>

        <div v-if="secretList">
          <div class="section-secrets">
            <div
              class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
            >
              Secret
            </div>

            <div class="flex items-center mt-2">
              <div
                class="w-40 px-2 text-sm leading-tight text-right text-white"
              >
                secretName
              </div>
              <div
                class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
                v-if="editorMode == 'edit'"
              >
                <select
                  class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none si-property input-bg-color-grey"
                  v-model="secretId"
                  data-cy="editor-property-viewer-secret-select"
                  @blur="saveSecretId"
                >
                  <option
                    v-for="secret in secretList"
                    :key="secret.value"
                    :value="secret.value"
                  >
                    {{ secret.label }}
                  </option>
                </select>
              </div>
              <div
                class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
                v-else
              >
                {{ secretName }}
              </div>
            </div>
          </div>
        </div>

        <div v-else>
          <div class="section-properties">
            <div
              class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
            >
              Properties
            </div>

            <div
              v-for="entityProperty in propertiesList"
              :key="entityProperty.id"
            >
              <div v-if="!entityProperty.hidden" class="flex flex-row">
                <div
                  class="w-full"
                  :style="propStyle(entityProperty)"
                  v-show="showPath(entityProperty)"
                >
                  <div
                    v-if="
                      repeated(entityProperty) &&
                      !propKind(entityProperty, 'select')
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

                  <div v-else-if="propKind(entityProperty, 'textArea')">
                    <PropTextArea
                      :entityProperty="entityProperty"
                      class="py-1"
                    />
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

        <div class="section-connections">
          <div
            class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
          >
            Connections
          </div>
        </div>

        <div class="section-configures">
          <div
            class="pt-1 pb-1 pl-8 mt-2 text-sm text-white align-middle property-section-bg-color"
          >
            Configures
          </div>

          <div>
            <div class="flex flex-col text-gray-500">
              <div
                class="flex flex-row w-full"
                v-for="successor in directSuccessors"
                :key="successor.id"
              >
                <div class="pl-8 pr-5">
                  {{ successor.objectType }} :: {{ objects[successor.id].name }}
                </div>
                <div
                  class="justify-end flex-grow pr-5 align-middle"
                  v-if="editorMode == 'edit'"
                >
                  <button
                    class="text-center focus:outline-none"
                    type="button"
                    @click="deleteSuccessorEdge(successor)"
                  >
                    <MinusSquareIcon size="1.25x" class=""></MinusSquareIcon>
                  </button>
                </div>
              </div>
            </div>
            <div
              class="flex justify-center pl-8 pr-5 text-gray-500 align-middle"
              v-if="editorMode == 'edit'"
            >
              <SiSelect
                name="newConfigures"
                :options="newConfiguresInputTypes"
                v-model="newConfigures"
                dataCy="editor-property-viewer-configures-select"
                size="xs"
              ></SiSelect>
              <button
                class="text-center focus:outline-none"
                type="button"
                :disabled="newConfigures === null"
                data-cy="editor-property-viewer-configures-button"
                @click="createNewConfigures"
              >
                <PlusSquareIcon size="1.25x" class=""></PlusSquareIcon>
              </button>
            </div>
          </div>
        </div>

        <div class="pb-24 section-resources">
          <div
            class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
          >
            Resources
          </div>
          <div class="flex flex-col w-full pl-5">
            <div>
              <Button2
                label="sync"
                icon="refresh"
                size="xs"
                @click.native="syncResource"
              />
            </div>
            <div class="text-gray-500" v-if="currentResource">
              <vue-json-pretty :data="currentResource.state" :deep="2" />
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
import PropTextArea from "./PropTextArea.vue";
import PropObject from "./PropObject.vue";
import PropNumber from "./PropNumber.vue";
import PropEnum from "./PropEnum.vue";
import PropMap from "./PropMap.vue";
import PropRepeated from "./PropRepeated.vue";
import PropBool from "./PropBool.vue";
import PropSelect from "./PropSelect.vue";
import SiSelect from "@/components/ui/SiSelect.vue";
import { Node, RegistryProperty } from "@/api/sdf/model/node";
import { Secret } from "@/api/sdf/model/secret";
//import { RegistryProperty } from "../../../../store/modules/node";

import { capitalCase } from "change-case";
import {
  EditIcon,
  InfoIcon,
  PlusSquareIcon,
  MinusSquareIcon,
} from "vue-feather-icons";
import _ from "lodash";
import VueJsonPretty from "vue-json-pretty";
import "vue-json-pretty/lib/styles.css";
import Button2 from "@/components/ui/Button2.vue";
import Tooltip from "@/components/ui/Tooltip.vue";

interface Data {
  collapsedPaths: (string | number)[][];
  nodeObjectName: string;
  newConfigures: string | null;
  secretId: string | undefined;
  tooltipOffset: number;
}

export default Vue.extend({
  name: "PropertyViewer",
  components: {
    PropText,
    PropTextArea,
    PropObject,
    PropNumber,
    PropEnum,
    PropMap,
    PropRepeated,
    PropBool,
    PropSelect,
    EditIcon,
    InfoIcon,
    PlusSquareIcon,
    MinusSquareIcon,
    SiSelect,
    VueJsonPretty,
    Button2,
    Tooltip,
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
      newConfigures: null,
      secretId: undefined,
      tooltipOffset: 7, //align the node info tooltip
    };
  },
  methods: {
    async syncResource() {
      await this.$store.dispatch("editor/syncResource");
    },
    async deleteSuccessorEdge(successor: Node) {
      await this.$store.dispatch("editor/deleteConfigures", successor);
    },
    async createNewConfigures() {
      await this.$store.dispatch(
        "editor/createNewConfigures",
        this.newConfigures,
      );
    },
    async updateObjectName() {
      await this.$store.dispatch("editor/entityNameSet", {
        value: this.nodeObjectName,
      });
    },
    togglePath(path: (string | number)[]) {
      if (
        _.find(this.collapsedPaths, (item) => {
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
      const collapsed = _.find(this.collapsedPaths, (path) => {
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
      const collapsed = _.find(this.collapsedPaths, (path) => {
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
    async saveSecretId(): Promise<void> {
      if (
        !_.isEqual(
          this.editObject?.properties.__baseline.secretId,
          this.secretId,
        )
      ) {
        console.log("saving now");
        await this.$store.dispatch("editor/entitySet", {
          path: ["secretId"],
          value: this.secretId,
        });
        await this.$store.dispatch("editor/syncCurrentResource");
      }
    },
  },
  computed: {
    typeName(): string {
      return capitalCase(this.selectedNode?.objectType || "unknown");
    },
    secretList(): { value: string | undefined; label: string }[] | undefined {
      const result = [{ label: "", value: undefined }];
      const secrets = this.$store.state.editor.secretList?.map((s: Secret) => {
        return { label: s.name, value: s.id };
      });
      if (secrets?.length > 0) {
        return result.concat(secrets);
      } else {
        return undefined;
      }
    },
    ...mapState({
      propertiesList: (state: any): RegistryProperty[] =>
        state.editor.propertyList,
      editorMode: (state: any): any => state.editor.mode,
      editObject: (state: any): any => state.editor.editObject,
      objects: (state: any): any => state.editor.objects,
      diff: (state: any): any => state.editor.diff,
      directSuccessors: (state: any): any => state.editor.directSuccessors,
      currentResource: (state: any): any => state.editor.currentResource,
      newConfiguresInputTypes: (state: any): any =>
        state.editor.newConfiguresInputTypes,
      secretName: (state: any): string | undefined => state.editor.secretName,
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
      let result = _.find(this.diff.entries, (diffEntry) => {
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
      if (this.editObject?.properties.__baseline.secretId) {
        this.secretId = _.cloneDeep(
          this.editObject?.properties.__baseline.secretId,
        );
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
