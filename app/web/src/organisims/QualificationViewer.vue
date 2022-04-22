<template>
  <div v-if="props.componentId" class="flex flex-col w-full">
    <div
      class="relative flex flex-row items-center justify-between h-10 pt-2 pb-2 pl-6 pr-6 text-white property-section-bg-color"
    >
      <div class="text-lg">
        Component ID {{ props.componentId }} Qualifications
      </div>

      <div class="flex">
        <button v-if="editMode" class="pl-1 focus:outline-none sync-button">
          <VueFeather
            type="refresh-cw"
            class="text-sm"
            :class="refreshButtonClasses"
            size="1.1em"
          />
        </button>
        <VueFeather
          v-else
          type="check-square"
          class="text-base text-sm text-gray-300"
          :class="refreshButtonClasses"
          size="1.1em"
        />
      </div>
    </div>

    <div class="flex flex-col mx-4 mt-2 border qualification-card">
      <div class="px-2 py-2 text-xs font-medium align-middle title">
        Qualification Checks
      </div>

      <div class="flex w-full h-full pt-2 pb-4 overflow-auto background-color">
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
                  v-if="qualificationResultQualified(q.result)"
                  type="smile"
                  class="text-green-300"
                  size="1.1em"
                />
                <!-- NOTE(nick): frown is slightly smaller than smile. It has been slightly boosted to match the latter. -->
                <VueFeather
                  v-else
                  type="frown"
                  class="text-xs error"
                  size="1.3em"
                />
              </div>
              <div v-else class="flex">
                <VueFeather type="square" class="text-gray-700" size="1.1em" />
              </div>

              <div
                v-if="q.title"
                class="flex ml-2 text-xs qualification-check-title"
              >
                {{ q.title }}
              </div>
              <div v-if="q.link" class="flex ml-2">
                <a target="_blank" :href="q.link">
                  <VueFeather type="info" class="info-button" size="1.1em" />
                </a>
              </div>
              <!-- NOTE(nick): We only render the button div if a description OR if a result exists
              in order to avoid user confusion. In essence, we want to ensure that we actually
              have something to show to the user.
              -->
              <div
                v-if="q.description || q.result"
                class="flex justify-end flex-grow pr-4"
              >
                <button
                  class="focus:outline-none"
                  @click="toggleShowDescription(q.name)"
                >
                  <VueFeather
                    v-if="showDescriptionMap[q.name] === true"
                    type="chevron-down"
                    size="1.1em"
                  />
                  <VueFeather v-else type="chevron-right" size="1.1em" />
                </button>
              </div>
            </div>

            <div
              v-if="showDescriptionMap[q.name] === true"
              class="flex flex-col w-full"
            >
              <!-- NOTE(nick): output is optional and can be empty. -->
              <div v-if="q.result" class="flex flex-col w-full">
                <div class="mt-1">
                  <QualificationOutput :result="q.result" :output="q.output" />
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
import * as Rx from "rxjs";
import { ComponentService } from "@/service/component";
import QualificationOutput from "./QualificationViewer/QualificationOutput.vue";
import VueFeather from "vue-feather";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import {
  Qualification,
  QualificationResult,
} from "@/api/sdf/dal/qualification";
import { VueLoading } from "vue-loading-template";
import { computed, ref, toRefs } from "vue";
import { ChangeSetService } from "@/service/change_set";
import { eventCheckedQualifications$ } from "@/observable/qualification";
import { system$ } from "@/observable/system";
//import { ListQualificationsResponse } from "@/service/component/list_qualifications";

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

// FIXME(nick): implement active state. Default to not starting for now.
const qualificationStarting = (_qualification_name: string) => {
  return false;
};

enum QualifiedState {
  Success,
  Failure,
  Unknown,
}

const currentQualifiedState = ref<QualifiedState>(QualifiedState.Unknown);
const getQualifiedState = (
  qualifications: Array<Qualification>,
): QualifiedState => {
  let empty = true;
  for (const q of qualifications) {
    if (q.result) {
      empty = false;
      if (!q.result.success) {
        return QualifiedState.Failure;
      }
    }
  }
  if (empty) {
    return QualifiedState.Unknown;
  }
  return QualifiedState.Success;
};

const refreshButtonClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (currentQualifiedState.value == QualifiedState.Success) {
    classes["error"] = false;
    classes["success"] = true;
    classes["unknown"] = false;
  } else if (currentQualifiedState.value === QualifiedState.Failure) {
    classes["error"] = true;
    classes["success"] = false;
    classes["unknown"] = false;
  } else {
    classes["error"] = false;
    classes["success"] = false;
    classes["unknown"] = true;
  }
  return classes;
});

const qualificationResultQualified = (result: QualificationResult) => {
  return result.success;
};

// Use a record to keep track of each qualification's description toggles. Maybe users want some
// boxes and not others. Who knows? I can't blame them. I just know that Maps aren't necessarily
// reactive, but Records are. By the gods, Vue!! TALOS GUIDE YOU.
const showDescriptionMap = ref<Record<string, boolean>>({
  allFieldsValid: false,
});

const toggleShowDescription = (name: string) => {
  showDescriptionMap.value[name] = !showDescriptionMap.value[name];
};

const populateShowDescription = (qualifications: Array<Qualification>) => {
  for (const q of qualifications) {
    if (!showDescriptionMap.value[q.name]) {
      showDescriptionMap.value[q.name] = false;
    }
  }
};

// We need an observable stream of props.componentId. We also want
// that stream to emit a value immediately (the first value, as well as all
// subsequent values)
const props = defineProps<{
  componentId: number;
}>();
const { componentId } = toRefs(props);
const componentId$ = fromRef<number>(componentId, { immediate: true });

const checkedQualifications$ = new Rx.ReplaySubject<true>();
checkedQualifications$.next(true); // We must fetch on setup
eventCheckedQualifications$
  .pipe(untilUnmounted)
  .subscribe(async (checkedQualificationId) => {
    const system = await Rx.firstValueFrom(system$);
    const data = checkedQualificationId?.payload.data;
    const sameComponent = props.componentId === data?.componentId;
    const sameSystem = (system?.id ?? -1) === data?.systemId;
    if (sameComponent && sameSystem) {
      checkedQualifications$.next(true);
    }
  });

const allQualifications = refFrom<Array<Qualification> | null>(
  Rx.combineLatest([componentId$, checkedQualifications$]).pipe(
    Rx.switchMap(([componentId]) => {
      // Reset qualified state before getting qualifications.
      currentQualifiedState.value = QualifiedState.Unknown;
      return ComponentService.listQualifications({
        componentId: componentId,
      });
    }),
    Rx.switchMap((reply) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
        return Rx.from([null]);
      } else {
        // Something something side effects... Let's rethink this someday.
        currentQualifiedState.value = getQualifiedState(reply);
        populateShowDescription(reply);
        return Rx.from([reply]);
      }
    }),
  ),
);
</script>

<style lang="scss" scoped>
$button-brightness: 1.1;

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

.property-section-bg-color {
  background-color: #292c2d;
}

.header-background {
  background-color: #1f2122;
}
</style>
