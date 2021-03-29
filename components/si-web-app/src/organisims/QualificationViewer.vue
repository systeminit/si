<template>
  <div class="flex flex-col w-full overflow-hidden" v-if="entity">
    <div
      class="relative flex flex-row items-center pt-2 pb-2 pl-6 pr-6 text-base text-white property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} qualifications
      </div>
    </div>
    <div
      v-if="schema && schema.qualifications"
      class="flex w-full pt-2 overflow-auto"
    >
      <div class="flex flex-col w-full">
        <div
          v-for="q in schema.qualifications"
          class="flex flex-col"
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
              <CheckSquareIcon
                class="text-green-300"
                v-if="qualificationResultQualified(q.name)"
              />
              <SquareIcon class="text-yellow-300" v-else />
            </div>
            <div class="flex" v-else>
              <SquareIcon class="text-gray-700" />
            </div>
            <div class="flex ml-2">
              {{ q.title }}
            </div>
            <div class="flex ml-2" v-if="q.link">
              <a target="_blank" :href="q.link">
                <ExternalLinkIcon size="1x" />
              </a>
            </div>
            <div class="flex justify-end flex-grow pr-4">
              <button @click="toggleDescription(q.name)">
                <MoreHorizontalIcon />
              </button>
            </div>
          </div>

          <div
            class="flex flex-col w-full pt-2 pl-8 pr-8"
            v-if="showDescription[q.name]"
          >
            <div class="flex flex-col w-full">
              <div class="flex flex-row w-full text-gray-300">
                Description
              </div>
              <div class="ml-4">{{ q.description }}</div>
            </div>
            <div
              class="flex flex-col w-full"
              v-if="hasQualificationResult(q.name)"
            >
              <div class="flex flex-row w-full text-gray-300">
                Output
              </div>
              <div class="flex ml-4">
                <CodeMirror
                  readOnly
                  lineWrapping
                  noHighlight
                  :value="qualificationResultOutput(q.name)"
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
import { RegistryEntry, registry } from "si-registry";
import {
  SquareIcon,
  CheckSquareIcon,
  MoreHorizontalIcon,
  ExternalLinkIcon,
} from "vue-feather-icons";
import _ from "lodash";

import CodeMirror from "@/molecules/CodeMirror.vue";
import { VueLoading } from "vue-loading-template";

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
    MoreHorizontalIcon,
    ExternalLinkIcon,
    CheckSquareIcon,
    CodeMirror,
    VueLoading,
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
        this.showDescription[name] = false;
      } else {
        this.showDescription[name] = true;
      }
    },
  },
});
</script>
