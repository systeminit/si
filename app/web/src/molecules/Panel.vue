<template>
  <div
    v-if="isVisible"
    class="flex flex-col w-full h-full"
    :class="panelClasses"
    @mouseenter="mouseEnter()"
    @mouseleave="mouseLeave()"
  >
    <div
      class="flex flex-row items-center w-full bg-black h-16"
      :class="panelMenuClasses"
    >
      <div class="flex justify-start">
        <div class="min-w-max w-200">
          <SiSelect
            id="selectPanelType"
            v-model="selectedPanelType"
            tooltip-text="Panel selector"
            :options="panelTypes"
            class="pl-2 w-32"
          />
        </div>
      </div>
      <div class="flex flex-row items-center flex-grow">
        <slot name="menuButtons"></slot>
      </div>
      <div class="flex flex-row items-center justify-end flex-grow">
        <div class="flex items-center h-full pr-2">
          <SiButtonIcon
            v-if="maximizedContainer && !maximizedFull"
            tooltip-text="Minimize Container"
            data-testid="minimize-container"
            @click="minimizeContainer"
          >
            <ChevronDownIcon />
          </SiButtonIcon>

          <SiButtonIcon
            v-if="
              !maximizedContainer &&
              !maximizedFull &&
              isMaximizedContainerEnabled
            "
            tooltip-text="Maximize Container"
            data-testid="maximize-container"
            @click="maximizeContainer"
          >
            <ChevronUpIcon />
          </SiButtonIcon>
        </div>
        <div class="flex items-center h-full pr-2">
          <SiButtonIcon
            v-if="maximizedFull"
            tooltip-text="Minimize Full"
            data-testid="minimize-full"
            @click="minimizeFull"
          >
            <ChevronDoubleDownIcon />
          </SiButtonIcon>

          <SiButtonIcon
            v-else
            tooltip-text="Maximize Full"
            data-testid="maximize-full"
            @click="maximizeFull"
          >
            <ChevronDoubleUpIcon />
          </SiButtonIcon>
        </div>
      </div>
    </div>
    <div class="flex flex-grow w-full h-full overflow-hidden">
      <slot name="content"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiSelect from "@/atoms/SiSelect.vue";
import { PanelMaximized, PanelType } from "@/organisims/PanelTree/panel_types";
import { LabelList } from "@/api/sdf/dal/label_list";
import { computed, onMounted, PropType, ref, watch } from "vue";
import {
  createPanelMaximizedContainerObservable,
  createPanelMaximizedFullObservable,
  restorePanelMaximizedContainerObservable,
  restorePanelMaximizedFullObservable,
} from "@/observable/editor";
import _ from "lodash";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import {
  ChevronUpIcon,
  ChevronDownIcon,
  ChevronDoubleUpIcon,
  ChevronDoubleDownIcon,
} from "@heroicons/vue/solid";

const props = defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialPanelType: {
    type: String as PropType<PanelType>,
    default: PanelType.Empty,
  },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: { type: Boolean, default: true },
  isMaximizedContainerEnabled: Boolean,
});

const currentSelectedPanelType = ref<PanelType>(props.initialPanelType);
const selectedPanelType = computed<PanelType>({
  get() {
    return currentSelectedPanelType.value;
  },
  set(value) {
    currentSelectedPanelType.value = value;
    changePanelType();
  },
});

const maximizedFull = ref(props.initialMaximizedFull);
const maximizedContainer = ref(props.initialMaximizedContainer);
const isActive = ref<boolean>(false);

const maximizedFull$ = createPanelMaximizedFullObservable(props.panelRef);
watch(maximizedFull, (mf) => {
  maximizedFull$.next(mf);
});
const maximizedContainer$ = createPanelMaximizedContainerObservable(
  props.panelRef,
);
watch(maximizedContainer, (mc) => {
  maximizedContainer$.next(mc);
});

onMounted(() => {
  const maximizedFullData = restorePanelMaximizedFullObservable(props.panelRef);
  if (!_.isNull(maximizedFullData)) {
    maximizedFull.value = maximizedFullData;
  }
  const maximizedContainerData = restorePanelMaximizedContainerObservable(
    props.panelRef,
  );
  if (!_.isNull(maximizedContainerData)) {
    maximizedContainer.value = maximizedContainerData;
  }
});

const panelTypes = computed<LabelList<PanelType>>(() => {
  return [
    {
      label: "Diagram",
      value: PanelType.Schematic,
    },
    {
      label: "Attribute",
      value: PanelType.Attribute,
    },
    {
      label: "Secret",
      value: PanelType.Secret,
    },
    {
      label: "Empty",
      value: PanelType.Empty,
    },
  ];
});

const emits = defineEmits([
  "change-panel",
  "panel-maximize-container",
  "panel-minimize-container",
  "panel-maximize-full",
  "panel-minimize-full",
]);

const changePanelType = () => {
  emits("change-panel", selectedPanelType.value);
};

const mouseEnter = () => {
  isActive.value = true;
};

const mouseLeave = () => {
  isActive.value = false;
};

function sizeEventData(): PanelMaximized {
  return {
    panelIndex: props.panelIndex,
    panelRef: props.panelRef,
    panelContainerRef: props.panelContainerRef,
  };
}

const maximizeContainer = () => {
  maximizedContainer.value = true;
  emits("panel-maximize-container", sizeEventData());
};
const minimizeContainer = () => {
  maximizedContainer.value = false;
  emits("panel-minimize-container", sizeEventData());
};
const maximizeFull = () => {
  maximizedFull.value = true;
  maximizedContainer.value = true;
  emits("panel-maximize-full", sizeEventData());
};
const minimizeFull = () => {
  maximizedFull.value = false;
  maximizedContainer.value = false;
  emits("panel-minimize-full", sizeEventData());
};

const panelClasses = computed(() => {
  let classes: Record<string, boolean> = {};
  classes["inactive-panel"] = !isActive.value;
  return classes;
});
const panelMenuClasses = computed(() => {
  let classes: Record<string, boolean> = {};
  classes["inactive-panel-menu"] = !isActive.value;
  return classes;
});
</script>

<style scoped>
div.inactive-panel-menu > * {
  filter: brightness(90%);
}

/*
div.active-panel {
  border: solid;
  border-color: #323536;
  border-width: 0.1em;
}

div.inactive-panel > * {
  filter: brightness(98%);
}
*/
</style>
