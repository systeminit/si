<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} qualifications
      </div>
    </div>
    <div
      v-if="schema"
      class="flex w-full h-full pt-2 overflow-auto background-color "
    >
      <div class="flex flex-col w-full">
        <div
          v-for="q in allQualifications"
          class="flex flex-col text-sm"
          :key="q.name"
        >
          <div class="flex flex-row items-center w-full pt-2 pl-4">
            <div class="flex" v-if="qualificationStarting(q.name)">
              <VueLoading
                class="inline-flex"
                type="cylon"
                :size="{ width: '24px', height: '24px' }"
              />
            </div>
            <div class="flex" v-else-if="hasQualificationResult(q.name)">
              <SmileIcon
                class="text-green-300"
                size="1x"
                v-if="qualificationResultQualified(q.name)"
              />
              <FrownIcon size="1.2x" class="text-xs error" v-else />
            </div>
            <div class="flex" v-else>
              <SquareIcon size="1x" class="text-gray-700" />
            </div>
            <div class="flex ml-2">
              {{ q.title }}
            </div>
            <div class="flex ml-2" v-if="q.link">
              <a target="_blank" :href="q.link">
                <InfoIcon class="info-button" size="1x" />
              </a>
            </div>
            <div class="flex justify-end flex-grow pr-4">
              <button
                class="focus:outline-none "
                @click="toggleDescription(q.name)"
              >
                <ChevronRightIcon size="1x" v-if="!showDescription[q.name]" />
                <ChevronDownIcon size="1x" v-else />
              </button>
            </div>
          </div>

          <div
            class="flex flex-col w-full pl-4 pr-2"
            v-if="showDescription[q.name]"
          >
            <div
              class="flex flex-col w-full"
              v-if="hasQualificationResult(q.name)"
            >
              <div class="mt-2 text-xs font-medium text-gray-300">
                Output
              </div>

              <div class="mt-1">
                <QualificationOutput
                  :kind="q.name"
                  :data="qualificationResultOutput(q.name)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Entity } from "@/api/sdf/model/entity";
import {
  Qualification,
  QualificationStart,
} from "@/api/sdf/model/qualification";
import {
  RegistryEntry,
  registry,
  allFieldsValidQualification,
  Qualification as SchemaQualification,
} from "si-registry";
import {
  SquareIcon,
  // CheckSquareIcon,
  ChevronRightIcon,
  InfoIcon,
  ChevronDownIcon,
  FrownIcon,
  SmileIcon,
} from "vue-feather-icons";
import _ from "lodash";

import { VueLoading } from "vue-loading-template";
import QualificationOutput from "@/organisims/QualificationViewer/QualificationOutput.vue";

interface Data {
  showDescription: Record<string, boolean>;
}

export default Vue.extend({
  name: "QualificationViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    qualifications: {
      type: Array as PropType<Qualification[]>,
    },
    starting: {
      type: Array as PropType<QualificationStart[]>,
    },
  },
  components: {
    SquareIcon,
    ChevronRightIcon,
    InfoIcon,
    // CheckSquareIcon,
    VueLoading,
    ChevronDownIcon,
    QualificationOutput,
    FrownIcon,
    SmileIcon,
  },
  data(): Data {
    const showDescription: Data["showDescription"] = {};
    if (registry[this.entity.entityType]) {
      if (_.isArray(registry[this.entity.entityType].qualifications)) {
        // @ts-ignore we just checked if this is an array; we're cool
        for (const q of registry[this.entity.entityType].qualifications) {
          showDescription[q.name] = false;
        }
      }
    }
    return {
      showDescription,
    };
  },
  computed: {
    allQualifications(): SchemaQualification[] {
      let quals = [allFieldsValidQualification];
      if (this.schema?.qualifications) {
        quals = _.concat(quals, this.schema.qualifications);
      }
      return quals;
    },
    schema(): RegistryEntry | null {
      if (registry[this.entity.entityType]) {
        return registry[this.entity.entityType];
      } else {
        return null;
      }
    },
  },
  methods: {
    qualificationStarting(name: string): boolean {
      const s = _.find(this.starting, ["start", name]);
      if (s) {
        return true;
      } else {
        return false;
      }
    },
    hasQualificationResult(name: string): boolean {
      const q = _.find(this.qualifications, ["name", name]);
      if (q) {
        return true;
      } else {
        return false;
      }
    },
    qualificationResultOutput(name: string): string {
      const q = _.find(this.qualifications, ["name", name]);
      if (q?.output) {
        return q.output;
      } else if (q?.error) {
        return q.error;
      } else {
        return "No output";
      }
    },
    qualificationResultKubevalOutput(name: string): string {
      const q = _.find(this.qualifications, ["name", name]);
      if (q?.output) {
        return q.output;
      } else if (q?.error) {
        return q.error;
      } else {
        return "No output";
      }
    },
    qualificationResultQualified(name: string): boolean {
      const q = _.find(this.qualifications, ["name", name]);
      if (q?.qualified) {
        return q.qualified;
      } else {
        return false;
      }
    },
    toggleDescription(name: string) {
      if (this.showDescription[name]) {
        Vue.set(this.showDescription, name, false);
      } else {
        Vue.set(this.showDescription, name, true);
      }
    },
  },
});
</script>

<style scoped>
.background-color {
  background-color: #151515;
}

.info-button {
  color: #8fc8ff;
}

.error {
  color: #ff8f8f;
}
</style>
