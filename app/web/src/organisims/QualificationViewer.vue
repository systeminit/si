<template>
  <div v-if="props.componentId" class="flex flex-col w-full">
    <div
      class="relative flex flex-row items-center justify-between h-10 pt-2 pb-2 pl-6 pr-6 text-white property-section-bg-color"
    >
      <div class="text-lg">
        Component number {{ props.componentId }}'s qualifications
      </div>

      <div class="flex">
        <button v-if="editMode" class="pl-1 focus:outline-none sync-button">
          <VueFeather
            ref="sync"
            type="refresh-cw"
            size="1.5rem"
            class="text-sm"
            :class="refreshButtonClasses()"
          />
        </button>
        <VueFeather
          v-else
          type="check-square"
          class="text-base text-sm text-gray-300"
          size="1.5rem"
          :class="refreshButtonClasses()"
        />
      </div>
    </div>

    <div class="flex flex-col mx-4 mt-2 border qualification-card">
      <div class="px-2 py-2 text-xs font-medium align-middle title">
        Qualification Checks
      </div>

      <div
        v-if="schema"
        class="flex w-full h-full pt-2 pb-4 overflow-auto background-color"
      >
        <div class="flex flex-col w-full">
          <div
            v-for="q in allQualifications"
            :key="q.name"
            class="flex flex-col py-1 mx-2 mt-2 text-sm border qualification-section"
          >
            <div class="flex flex-row items-center w-full pl-4 my-1">
              <div v-if="qualificationStarting(q.name)" class="flex">
                <VueLoading
                  class="inline-flex"
                  type="cylon"
                  :size="{ width: '14px', height: '14px' }"
                />
              </div>
              <div v-else-if="q.result" class="flex">
                <VueFeather
                  v-if="qualificationResultQualified(q)"
                  type="smile"
                  class="text-green-300"
                  size="1.5rem"
                />
                <VueFeather
                  v-else
                  type="frown"
                  class="text-xs error"
                  size="1.5rem"
                />
              </div>
              <div v-else class="flex">
                <VueFeather type="square" class="text-gray-700" size="1.5rem" />
              </div>

              <div
                v-if="q.title"
                class="flex ml-2 text-xs qualification-check-title"
              >
                {{ q.title }}
              </div>
              <div v-if="q.link" class="flex ml-2">
                <a target="_blank" :href="q.link">
                  <VueFeather type="link" class="info-button" size="1.5rem" />
                </a>
              </div>
              <div class="flex justify-end flex-grow pr-4">
                <button
                  class="focus:outline-none"
                  @click="setShowDescription(q.description)"
                >
                  <VueFeather
                    v-if="showDescription"
                    type="chevron-up"
                    size="1.5rem"
                  />
                  <VueFeather v-else type="chevron-down" size="1.5rem" />
                </button>
              </div>
            </div>

            <div
              v-if="showDescription && q.description"
              class="flex flex-col w-full"
            >
              <div v-if="q.result" class="flex flex-col w-full">
                <div class="mt-1">
                  <QualificationOutput :result="q.result" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentService } from "@/service/component";
import QualificationOutput from "./QualificationViewer/QualificationOutput.vue";
import VueFeather from "vue-feather";
import { refFrom } from "vuse-rx";
import { from, switchMap } from "rxjs";
import { GlobalErrorService } from "@/service/global_error";
import { Qualification } from "@/api/sdf/dal/qualification";
import { VueLoading } from "vue-loading-template";
import { ref } from "vue";
import { ChangeSetService } from "@/service/change_set";
//import { ListQualificationsResponse } from "@/service/component/list_qualifications";

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

// TODO(nick): replace mock data and functions.
const schema = true;

// FIXME(nick): implement active state.
function isQualifying() {
  return false;
}

// FIXME(nick): implement active state.
function qualificationStarting(_qualification_name: string) {
  return false;
}

function qualificationResultQualified(qualification: Qualification) {
  if (qualification.result) {
    return qualification.result?.success;
  }
  return false;
}

function isQualified(qualifications: Array<Qualification>): boolean {
  qualifications.forEach((q) => {
    // Do nothing if the result is qualified or does not exist.
    if (q.result && !q.result.success) {
      return false;
    }
  });

  // All results are qualified, do not have an attached result, or a mixture of the two.
  return true;
}

const showDescription = ref<boolean>(false);
function setShowDescription(description: string) {
  showDescription.value = description !== "";
}

const props = defineProps<{
  componentId: number;
}>();

const allQualifications = refFrom<Array<Qualification>>(
  ComponentService.listQualifications({
    componentId: props.componentId,
  }).pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response]);
      }
    }),
  ),
);

function refreshButtonClasses(): Record<string, any> {
  let classes: Record<string, any> = {};
  if (allQualifications.value && allQualifications.value.length > 0) {
    if (isQualifying()) {
      classes["animation"] = "spin";
      classes["success"] = false;
      classes["error"] = false;
      classes["qualifying"] = true;
    }

    if (isQualified(allQualifications.value)) {
      classes["success"] = true;
    } else {
      classes["error"] = true;
    }
  } else {
    classes["unknown"] = true;
  }
  return classes;
}
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

/*.sync-button:hover {*/
/*  filter: brightness($button-brightness);*/
/*}*/

.sync-button:focus {
  outline: none;
}

/*.sync-button:active {*/
/*  filter: saturate(1.5) brightness($button-brightness);*/
/*}*/
</style>
