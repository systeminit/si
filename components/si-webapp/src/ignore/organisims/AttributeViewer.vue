<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }}
      </div>

      <div class="ml-2 text-base">
        <CheckSquareIcon size="1x" :class="qualificationStatus" />
      </div>

      <div class="ml-2 text-base">
        <BoxIcon size="1x" :class="resourceHealthStatus" />
      </div>

      <div
        class="flex flex-row items-center justify-end flex-grow h-full text-xs text-center"
      >
        <div v-if="diff && diff.length > 0" class="flex flex-row items-center">
          <EditIcon size="1x" class="gold-bars-icon" />
          <div v-if="diff" class="ml-1 text-center">{{ diff.length }}</div>
        </div>
      </div>
    </div>

    <div class="flex flex-col w-full overflow-auto scrollbar">
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
      <template v-if="editFields && treeOpenState">
        <EditFields
          :entity="entity"
          :editMode="editMode"
          :editFields="editFields"
          :systemId="systemId"
          :backgroundColors="backgroundColors"
          :treeOpenState="treeOpenState"
          :diff="diff"
          @toggle-path="togglePath"
          @set-tree-open-state="setTreeOpenState"
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

import { EditIcon, CheckSquareIcon, BoxIcon } from "vue-feather-icons";
import NameField from "@/organisims/AttributeViewer/NameField.vue";
import EditFields from "@/organisims/AttributeViewer/EditFields.vue";
import { Entity } from "@/api/sdf/model/entity";
import { Resource, ResourceHealth } from "@/api/sdf/model/resource";
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
  treeOpenState: {
    [pathKey: string]: boolean;
  };
}

export default Vue.extend({
  name: "AttributeViewer",
  components: {
    EditIcon,
    NameField,
    EditFields,
    CheckSquareIcon,
    BoxIcon,
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
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  data(): Data {
    return {
      treeOpenState: {},
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
      let longestProp = 50;
      // BUG: There is a bug here - this will increase
      //      the longestProp number if there is a clearly nested, non-array
      //      object depth greater than 50. But it won't deal with array's
      //      at all. If we need to figure out how to deal with nested arrays
      //      of objects that are more than 50 levels deep, this code will
      //      need to be fixed.
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
    resourceHealthStatus(): Record<string, any> {
      let style: Record<string, any> = {};

      if (this.resource) {
        if (this.resource.health == ResourceHealth.Ok) {
          style["ok"] = true;
        } else if (this.resource.health == ResourceHealth.Warning) {
          style["warning"] = true;
        } else if (this.resource.health == ResourceHealth.Error) {
          style["error"] = true;
        } else if (this.resource.health == ResourceHealth.Unknown) {
          style["unknown"] = true;
        } else {
          style["unknown"] = true;
        }
      } else {
        style["unknown"] = true;
      }
      return style;
    },
    qualificationStatus(): Record<string, any> {
      let style: Record<string, any> = {};

      if (this.qualifications.length > 0) {
        if (this.isQualifying()) {
          style["unknown"] = true;
        } else {
          if (this.qualificationResultQualified) {
            style["ok"] = true;
          } else {
            style["error"] = true;
          }
        }
      } else {
        style["unknown"] = true;
      }

      return style;
    },
  },
  methods: {
    togglePath(pathKey: string) {
      if (this.treeOpenState.hasOwnProperty(pathKey)) {
        Vue.set(this.treeOpenState, pathKey, !this.treeOpenState[pathKey]);
      }
    },
    setTreeOpenState(entry: { key: string; value: boolean }) {
      Vue.set(this.treeOpenState, entry.key, entry.value);
      this.updateTreeOpenState();
    },
    updateTreeOpenState() {
      if (this.entity) {
        const headerEditFields = this.entity
          .editFields()
          .filter(
            editField =>
              editField.type == "object" && editField.widgetName == "header",
          );

        for (const editField of headerEditFields) {
          const key = editField.path.join("::");
          if (!this.treeOpenState.hasOwnProperty(key)) {
            Vue.set(this.treeOpenState, key, false);
          }
        }

        for (const op of this.entity.ops) {
          // Find all parent header paths from the `EditField`, sorted by
          // hierarchy
          const pathKeys = Object.keys(this.treeOpenState)
            .filter(pathKey =>
              this.entity.subPath(op.path, pathKey.split("::")),
            )
            .sort();

          // Open each parent header, starting at the top of the hierarchy
          for (const pathKey of pathKeys) {
            Vue.set(this.treeOpenState, pathKey, true);
          }
        }
      } else {
        this.treeOpenState = {};
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
    isQualifying(): boolean {
      if (this.starting.length > 0) {
        return true;
      } else {
        return false;
      }
    },
  },
  watch: {
    entity: {
      deep: true,
      immediate: true,
      handler() {
        this.updateTreeOpenState();
      },
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

.ok {
  color: #86f0ad;
}

.warning {
  color: #f0d286;
}

.error {
  color: #f08686;
}

.unknown {
  color: #5b6163;
}

.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}
</style>
