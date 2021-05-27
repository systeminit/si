<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg ">
        {{ entity.entityType }}
      </div>

      <div class="ml-2 text-xs">
        <div v-if="schema">
          <div class="flex flex-col w-full">
            <div>
              <div class="flex" v-if="hasQualificationResult">
                <CheckCircleIcon
                  class="verification-passed"
                  size="1x"
                  v-if="qualificationResultQualified"
                />
                <AlertCircleIcon size="1x" class="verification-failed" v-else />
              </div>
              <div class="flex" v-else>
                <CircleIcon size="1x" class="verification-unknown" />
              </div>
            </div>
          </div>
        </div>
        <CircleIcon size="1x" class="verification-unknown" v-else />
      </div>

      <div
        class="flex flex-row items-center items-end justify-end flex-grow h-full text-xs text-center"
      >
        <div v-if="diff && diff.length > 0" class="flex flex-row items-center">
          <EditIcon size="1x" class="gold-bars-icon" />
          <div v-if="diff" class="ml-1 text-center">{{ diff.length }}</div>
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

import {
  EditIcon,
  CheckCircleIcon,
  AlertCircleIcon,
  CircleIcon,
} from "vue-feather-icons";
import NameField from "@/organisims/AttributeViewer/NameField.vue";
import EditFields from "@/organisims/AttributeViewer/EditFields.vue";
import { Entity } from "@/api/sdf/model/entity";
import { editMode$, system$ } from "@/observables";
import { Diff } from "@/api/sdf/model/diff";
import {
  Qualification,
  QualificationStart,
} from "@/api/sdf/model/qualification";

import _ from "lodash";
import { EditField } from "si-entity";
import { RegistryEntry, registry } from "si-registry";

interface Data {
  closedPaths: string[][];
}

export default Vue.extend({
  name: "AttributeViewer",
  components: {
    EditIcon,
    NameField,
    EditFields,
    CheckCircleIcon,
    AlertCircleIcon,
    CircleIcon,
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
    qualifications: {
      type: Array as PropType<Qualification[]>,
    },
    starting: {
      type: Array as PropType<QualificationStart[]>,
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
    schema(): RegistryEntry | null {
      if (registry[this.entity.entityType]) {
        return registry[this.entity.entityType];
      } else {
        return null;
      }
    },
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
    hasQualificationResult(): boolean {
      if (this.qualifications && this.qualifications.length > 0) {
        return true;
      } else {
        return false;
      }
    },
    qualificationResultQualified(): boolean {
      const q = _.find(this.qualifications, ["qualified", false]);
      if (q) {
        return false;
      } else {
        return true;
      }
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
    qualificationStarting(name: string): boolean {
      const s = _.find(this.starting, ["start", name]);
      if (s) {
        return true;
      } else {
        return false;
      }
    },
  },
});
</script>

<style scoped>
.verification-passed {
  color: #44e368;
}

.verification-failed {
  color: #e35f44;
}

.verification-unknown {
  color: #707070;
}
</style>
