<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 pt-2 pb-2 pl-6 pr-6 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }}
      </div>
      <div
        class="flex flex-row items-center items-end justify-end flex-grow h-full text-xs text-center"
      >
        <div v-if="diff">
          <EditIcon size="1x" class="mr-1 gold-bars-icon" />
        </div>
        <div v-if="diff" class="text-center">Edit Count: {{ diff.length }}</div>
        <div class="ml-2">
          <Tooltip class="inline" alignRight :offset="0.5" sticky>
            <InfoIcon size="1x" class="inline mr-1" />
            <template v-slot:tooltip>
              <div class="flex flex-col text-sm text-gray-400">
                <div class="pl-2">
                  {{ entity.nodeId }}
                </div>
                <div class="pl-2">
                  {{ entity.id }}
                </div>
              </div>
            </template>
          </Tooltip>
        </div>
      </div>
    </div>

    <div class="flex flex-col w-full overflow-auto overscroll-none">
      <NameField
        :entity="entity"
        :editMode="editMode"
        :path="[]"
        :systemId="systemId"
        :forName="true"
        :diff="diff"
      />
      <div
        class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
        v-if="editFields && editFields.length"
      >
        Properties
      </div>
      <template v-if="editFields">
        <EditFields
          :entity="entity"
          :editMode="editMode"
          :editFields="editFields"
          :systemId="systemId"
          :backgroundColors="backgroundColors"
          :closedPaths="closedPaths"
          :diff="diff"
          @toggle-path="togglePath"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.gold-bars-icon {
  color: #ce7f3e;
}

.property-section-bg-color {
  background-color: #292c2d;
}
</style>

<script lang="ts">
import Vue, { PropType } from "vue";

import Tooltip from "@/atoms/Tooltip.vue";
import { EditIcon, InfoIcon } from "vue-feather-icons";
import NameField from "@/organisims/AttributeViewer/NameField.vue";
import EditFields from "@/organisims/AttributeViewer/EditFields.vue";
import { Entity } from "@/api/sdf/model/entity";
import { editMode$, system$ } from "@/observables";
import { Diff } from "@/api/sdf/model/diff";

import _ from "lodash";
import { EditField } from "si-entity";
import { RegistryEntry } from "si-registry";

interface Data {
  closedPaths: string[][];
}

export default Vue.extend({
  name: "AttributeViewer",
  components: {
    Tooltip,
    InfoIcon,
    EditIcon,
    NameField,
    EditFields,
  },
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    diff: {
      type: Array as PropType<Diff>,
      required: true,
    },
  },
  data(): Data {
    return {
      closedPaths: [],
    };
  },
  subscriptions() {
    return {
      editMode: editMode$,
      system: system$,
    };
  },
  computed: {
    schemaProperties(): RegistryEntry["properties"] {
      if (this.entity) {
        return this.entity.schema().properties;
      } else {
        return [];
      }
    },
    editFields(): ReturnType<Entity["editFields"]> | undefined {
      if (this.entity) {
        return this.entity.editFields();
      } else {
        return undefined;
      }
    },
    systemId(): string {
      // @ts-ignore
      if (this.system) {
        // @ts-ignore
        return this.system.id;
      } else {
        return "baseline";
      }
    },
    backgroundColors(): number[][] {
      let longestProp = 0;
      if (this.editFields) {
        for (const field of this.editFields) {
          if (field.path) {
            if (field.path.length > longestProp) {
              longestProp = field.path.length;
            }
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
  },
  methods: {
    showFieldForWidget(widget: string, editField: EditField): boolean {
      let closedByPath = _.find(this.closedPaths, p =>
        _.isEqual(p, editField.path.slice(0, p.length)),
      );
      if (closedByPath) {
        if (editField.widgetName == "header") {
          let isHeader = _.find(this.closedPaths, p =>
            _.isEqual(p, editField.path),
          );
          if (isHeader) {
            return true;
          } else {
            return false;
          }
        } else {
          return false;
        }
      }
      return editField.widgetName == widget;
    },
    togglePath(path: string[]) {
      let pathExists = _.find(this.closedPaths, p => _.isEqual(p, path));
      if (pathExists) {
        this.closedPaths = _.filter(this.closedPaths, p => !_.isEqual(p, path));
      } else {
        this.closedPaths.push(path);
      }
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
  },
});
</script>
