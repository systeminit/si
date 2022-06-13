<template>
  <div v-if="props.componentId" class="flex flex-col w-full">
    <div
      class="flex justify-between h-10 pt-2 pr-6 pl-6 property-section-bg-color"
    >
      <div class="text-lg">
        Component ID {{ props.componentId }} Qualifications
      </div>

      <div class="flex">
        <SiIcon :tooltip-text="qualificationTooltip">
          <CheckCircleIcon :class="qualificationStatusClasses" />
        </SiIcon>
        <SiButtonIcon
          v-if="editMode"
          class="ml-2"
          tooltip-text="Re-run qualifications"
          @click="runQualification"
        >
          <RefreshIcon :class="refreshButtonClasses" />
        </SiButtonIcon>
      </div>
    </div>

    <div
      v-if="editingQualificationId"
      class="flex flex-col w-full h-full max-h-[90%]"
    >
      <QualificationEditor
        :prototype-id="editingQualificationId"
        @close="editingQualificationId = undefined"
      />
    </div>

    <div class="flex flex-col mx-4 border qualification-card">
      <div
        v-if="!editingQualificationId"
        class="flex justify-between h-10 mt-2 pt-2 pr-6 pl-6"
      >
        <div class="px-2 py-2 text-xs font-medium align-middle title">
          Qualification Checks
        </div>

        <div class="flex">
          <SiButtonIcon
            v-if="editMode"
            class="mr-2 mt-1 text-green-300"
            tooltip-text="Create new qualification function"
            @click="createQualification"
          >
            <PlusCircleIcon />
          </SiButtonIcon>
          <span
            class="cursor-pointer text-green-300 underline"
            @click="createQualification"
            >add qualification</span
          >
        </div>
      </div>

      <div class="flex w-full h-full pt-2 pb-4 overflow-auto background-color">
        <div class="flex flex-col w-full">
          <div
            v-for="q in allQualifications"
            :key="q.title"
            class="flex flex-col py-1 mx-2 mt-2 text-sm border qualification-section"
          >
            <div
              :v-tooltip="q.description"
              class="flex flex-row items-center w-full pl-4 my-1"
            >
              <div v-if="qualificationStarting(q.title)" class="flex">
                <VueLoading
                  class="inline-flex"
                  type="cylon"
                  :size="{ width: '14px', height: '14px' }"
                />
              </div>
              <div v-else-if="q.result" class="flex">
                <SiIcon
                  v-if="qualificationResultQualified(q.result)"
                  tooltip-text="Qualification succeeded"
                >
                  <EmojiHappyIcon class="text-green-300" />
                </SiIcon>
                <SiIcon v-else tooltip-text="Qualification failed">
                  <EmojiSadIcon class="error" />
                </SiIcon>
              </div>
              <div v-else class="flex">
                <SiIcon tooltip-text="No qualification result found">
                  <RefreshIconOutline class="text-gray-700" />
                </SiIcon>
              </div>

              <div
                v-if="q.title"
                class="flex ml-2 mr-2 text-xs qualification-check-title"
              >
                {{ q.title }}
              </div>
              <SiLink v-if="q.link" :blank-target="true" :uri="q.link">
                <SiIcon tooltip-text="Go to docs">
                  <InformationCircleIcon />
                </SiIcon>
              </SiLink>

              <div class="flex justify-end flex-grow pr-4">
                <SiButtonIcon
                  v-if="q.prototypeId && !editingQualificationId"
                  class="focus:outline-none mr-2"
                  tooltip-text="Edit qualification function"
                  @click="editingQualificationId = q.prototypeId"
                >
                  <PencilAltIcon />
                </SiButtonIcon>

                <SiButtonIcon
                  v-if="q.description || q.result"
                  class="focus:outline-none"
                  :tooltip-text="
                    showDescriptionMap[q.title]
                      ? 'Show description'
                      : 'Hide description'
                  "
                  @click="toggleShowDescription(q.title)"
                >
                  <ChevronDownIcon v-if="showDescriptionMap[q.title]" />
                  <ChevronUpIcon v-else />
                </SiButtonIcon>
              </div>
            </div>

            <div
              v-if="showDescriptionMap[q.title] === true"
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
import { fromRef, refFrom, untilUnmounted } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import {
  Qualification,
  QualificationResult,
} from "@/api/sdf/dal/qualification";
import { VueLoading } from "vue-loading-template";
import { computed, ref, toRefs } from "vue";
import { ChangeSetService } from "@/service/change_set";
import { QualificationService } from "@/service/qualification";
import { eventCheckedQualifications$ } from "@/observable/qualification";
import { system$ } from "@/observable/system";
import SiLink from "@/atoms/SiLink.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import QualificationEditor from "@/organisims/QualificationEditor.vue";
import SiIcon from "@/atoms/SiIcon.vue";
import { RefreshIcon, CheckCircleIcon } from "@heroicons/vue/solid";
import {
  InformationCircleIcon,
  RefreshIcon as RefreshIconOutline,
  EmojiSadIcon,
  EmojiHappyIcon,
  ChevronDownIcon,
  PencilAltIcon,
  ChevronUpIcon,
  PlusCircleIcon,
} from "@heroicons/vue/outline";
import { toast$ } from "@/observable/toast";
//import { ListQualificationsResponse } from "@/service/component/list_qualifications";

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const editingQualificationId = ref<number | undefined>(undefined);

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
const currentQualifiedAnimate = ref<boolean>(false);

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

const createQualification = async () => {
  const system = await Rx.firstValueFrom(system$);

  QualificationService.create({
    componentId: props.componentId,
    systemId: system?.id,
  }).subscribe((reply) => {
    if (reply.error) {
      GlobalErrorService.set(reply);
    } else {
      editingQualificationId.value = reply.prototypeId;
    }
  });
};

const runQualification = () => {
  currentQualifiedAnimate.value = true;
  ComponentService.checkQualifications({
    componentId: props.componentId,
  }).subscribe((reply) => {
    currentQualifiedAnimate.value = false;
    if (
      reply.error?.statusCode === 404 &&
      reply.error?.message === "invalid visibility"
    ) {
      return Rx.from([null]);
    } else if (reply.error) {
      GlobalErrorService.set(reply);
    } else if (!reply.success) {
      GlobalErrorService.set({
        error: {
          statusCode: 42,
          code: 42,
          message: "Qualification check failed silently",
        },
      });
    }
  });
};

const qualificationTooltip = computed(() => {
  if (currentQualifiedAnimate.value === true) {
    return "Qualification is running";
  } else if (currentQualifiedState.value === QualifiedState.Success) {
    return "Qualification succeeded";
  } else if (currentQualifiedState.value === QualifiedState.Failure) {
    return "Qualification failed";
  } else {
    return "Qualification is unknown";
  }
});

const refreshButtonClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (currentQualifiedAnimate.value) {
    classes["animate-spin"] = true;
    classes["transform"] = true;
    classes["rotate-180"] = true;
  }
  return classes;
});

const qualificationStatusClasses = computed(() => {
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
  if (currentQualifiedAnimate.value) {
    classes["success"] = false;
    classes["error"] = false;
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
    if (!showDescriptionMap.value[q.title]) {
      showDescriptionMap.value[q.title] = false;
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

const scheduledToasts = ref<number[]>([]);

componentId$.pipe(untilUnmounted).subscribe(() => {
  scheduledToasts.value = [];
  editingQualificationId.value = undefined;
});

const checkedQualifications$ = new Rx.ReplaySubject<true>();
checkedQualifications$.next(true); // We must fetch on setup
eventCheckedQualifications$
  .pipe(untilUnmounted)
  .subscribe(async (checkedQualificationId) => {
    const system = await Rx.firstValueFrom(system$);
    const data = checkedQualificationId?.payload.data;
    const sameComponent = props.componentId === data?.componentId;
    const sameSystem = (system?.id ?? -1) === data?.systemId;
    if (data && sameComponent && sameSystem) {
      scheduledToasts.value.push(data.prototypeId);
      checkedQualifications$.next(true);
    }
  });

const allQualifications = refFrom<Array<Qualification> | null>(
  Rx.combineLatest([componentId$, system$, checkedQualifications$]).pipe(
    Rx.switchMap(([componentId]) => {
      // Reset qualified state before getting qualifications.
      currentQualifiedState.value = QualifiedState.Unknown;
      return ComponentService.listQualifications({
        componentId: componentId,
      });
    }),
    Rx.switchMap((reply) => {
      if (
        reply.error?.statusCode === 404 &&
        reply.error?.message === "invalid visibility"
      ) {
        return Rx.from([null]);
      } else if (reply.error) {
        GlobalErrorService.set(reply);
        return Rx.from([null]);
      } else {
        for (const prototypeId of scheduledToasts.value) {
          const qual = reply.find((q) => q.prototypeId === prototypeId);
          if (qual?.result) {
            toast$.next({
              id: `qualification-prototype-${qual.prototypeId}`,
              success: qual.result.success,
              title: `Qualification ${
                qual.result.success ? "succeeded" : "failed"
              }`,
              subtitle: qual.title,
              message: (qual.output ?? []).map((o) => o.line).join("\n"),
            });
          }
        }
        scheduledToasts.value = [];

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

.run-button {
  color: #a8cc5f;
}

.run-button:hover {
  filter: brightness($button-brightness);
}

.run-button:focus {
  outline: none;
}

.run-button:active {
  filter: saturate(1.5) brightness($button-brightness);
}

.property-section-bg-color {
  background-color: #292c2d;
}

.header-background {
  background-color: #1f2122;
}
.run-button-invert {
  transform: scaleX(-1);
}
</style>
