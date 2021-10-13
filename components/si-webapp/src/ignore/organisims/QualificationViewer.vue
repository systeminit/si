<template>
  <div class="flex flex-col w-full overflow-auto" v-if="entity">
    <div
      class="relative flex flex-row items-center justify-between h-10 pt-2 pb-2 pl-6 pr-6 text-white property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} qualifications
      </div>

      <div class="flex">
        <button
          class="pl-1 focus:outline-none sync-button"
          v-if="editMode"
          @click="checkQualifications()"
        >
          <RefreshCwIcon
            ref="sync"
            size="1.1x"
            class="text-sm"
            :class="refreshButtonClasses()"
          />
        </button>
        <CheckSquareIcon
          size="1x"
          class="text-base"
          :class="refreshButtonClasses()"
          v-else
        />
      </div>
    </div>

    <div class="flex flex-col mx-4 mt-2 border qualification-card">
      <div class="px-2 py-2 text-xs font-medium align-middle title">
        Qualification Checks
      </div>

      <div
        v-if="schema"
        class="flex w-full h-full pt-2 pb-4 overflow-auto background-color "
      >
        <div class="flex flex-col w-full">
          <div
            v-for="q in allQualifications"
            class="flex flex-col py-1 mx-2 mt-2 text-sm border qualification-section"
            :key="q.name"
          >
            <div class="flex flex-row items-center w-full pl-4 my-1">
              <div class="flex" v-if="qualificationStarting(q.name)">
                <VueLoading
                  class="inline-flex"
                  type="cylon"
                  :size="{ width: '14px', height: '14px' }"
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
              <div class="flex ml-2 text-xs qualification-check-title">
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

            <div class="flex flex-col w-full " v-if="showDescription[q.name]">
              <div
                class="flex flex-col w-full"
                v-if="hasQualificationResult(q.name)"
              >
                <div class="mt-1 ">
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
  CheckSquareIcon,
  ChevronRightIcon,
  InfoIcon,
  ChevronDownIcon,
  FrownIcon,
  SmileIcon,
  RefreshCwIcon,
} from "vue-feather-icons";
import _ from "lodash";

import { VueLoading } from "vue-loading-template";
import QualificationOutput from "@/organisims/QualificationViewer/QualificationOutput.vue";
import {
  editMode$,
  editSession$,
  system$,
  workspace$,
  changeSet$,
} from "@/observables";
import { AttributeDal } from "@/api/sdf/dal/attributeDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

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
    CheckSquareIcon,
    VueLoading,
    ChevronDownIcon,
    QualificationOutput,
    FrownIcon,
    SmileIcon,
    RefreshCwIcon,
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
    isQualifying(): boolean {
      if (this.starting.length > 0) {
        return true;
      } else {
        return false;
      }
    },
    isQualified(): boolean {
      const q = _.find(this.qualifications, ["qualified", false]);
      if (q) {
        return false;
      } else {
        return true;
      }
    },
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
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      changeSet: changeSet$,
      editSession: editSession$,
      system: system$,
      workspace: workspace$,
    };
  },
  methods: {
    async checkQualifications(this: any): Promise<void> {
      if (this.changeSet && this.editSession && this.system && this.workspace) {
        this.animateSyncButton();
        let result = await AttributeDal.checkQualifications({
          entityId: this.entity.id,
          changeSetId: this.changeSet.id,
          editSessionId: this.editSession.id,
          systemId: this.system.id,
          workspaceId: this.workspace.id,
        });
        if (result.error) {
          emitEditorErrorMessage(result.error.message);
        }
      }
    },
    // isQualifying(): boolean {
    //   if (this.starting.length > 0) {
    //     return true;
    //   } else {
    //     return false;
    //   }
    // },
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
    animateSyncButton() {
      // const button = this.$refs.sync as HTMLElement;
      // if (button) {
      //   button.animate(
      //     [{ transform: "rotate(0deg)" }, { transform: "rotate(180deg)" }],
      //     {
      //       duration: 225,
      //       easing: "ease-out",
      //     },
      //   );
      // }
    },
    refreshButtonClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (this.qualifications.length > 0) {
        if (this.isQualifying) {
          classes["animate-spin"] = true;
          classes["success"] = false;
          classes["error"] = false;
          classes["qualifying"] = true;
        }

        if (this.isQualified) {
          classes["success"] = true;
        } else {
          classes["error"] = true;
        }
      } else {
        classes["unknown"] = true;
      }
      return classes;
    },
  },
});
</script>

<style scoped>
.background-color {
  background-color: #151515;
}

.title {
  background-color: #1f2122;
  color: #e9f2fe;
}

.qualification-card {
  border-color: #1f2122;
}

.qualification-check-title {
  color: #d4d4d4;
}

.qualification-section {
  border-color: #2d3032;
}

.info-button {
  color: #8fc8ff;
}

.success {
  color: #86f0ad;
}

.error {
  color: #f08686;
}

.unknown {
  color: #bbbbbb;
}

.qualifying {
  color: #bbbbbb;
}

.sync-button {
  color: #a8cc5f;
}

.sync-button:hover {
  filter: brightness($button-brightness);
}

.sync-button:focus {
  outline: none;
}

.sync-button:active {
  filter: saturate(1.5) brightness($button-brightness);
}
</style>
