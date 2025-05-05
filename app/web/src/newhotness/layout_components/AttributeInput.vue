<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <label class="pl-xs flex flex-row items-center relative">
    <span>{{ displayName }}</span>
    <template v-if="maybeOptions.hasOptions"> </template>
    <valueForm.Field name="value">
      <template #default="{ field }">
        <input
          ref="anchorRef"
          class="block w-72 ml-auto text-white bg-black border-2 border-neutral-300 disabled:bg-neutral-900"
          type="text"
          :value="field.state.value"
          :disabled="wForm.bifrosting.value"
          @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
          @blur="blur"
          @focus="focus"
          @keydown.esc.stop.prevent="hideOptions"
        />
      </template>
    </valueForm.Field>
    <!-- `relative` on label just to "float" this loader above the form input -->
    <Icon
      v-if="wForm.bifrosting.value"
      class="absolute right-2xs"
      name="loader"
      size="sm"
      tone="action"
    />
  </label>

  <!-- `relative` on label means we need a `z-100` here to get this above everyone -->
  <div
    v-show="maybeOptions.hasOptions && showOptions"
    ref="optionRef"
    class="absolute w-1/2 h-[12rem] self-end bg-neutral-500 z-100"
    :style="optionPos"
  >
    <ol class="scrollable h-full">
      <li class="p-xs">
        <!-- TODO make it a fuzzy filter -->
        <input
          type="text"
          class="text-white bg-black border-2 border-neutral-300 w-full block"
          placeholder="filter..."
          @focus="() => (filterFocus = true)"
          @blur="blurFilter"
        />
      </li>
      <li
        v-for="option in maybeOptions.options"
        :key="option.value"
        class="cursor-pointer p-xs hover:bg-black"
        @mousedown="select(option)"
      >
        {{ option.label }}
      </li>
    </ol>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import { BifrostAttributeTree } from "@/workers/types/dbinterface";
import {
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import { attributeEmitter } from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";

const props = defineProps<{
  attributeTree: BifrostAttributeTree;
  displayName: string;
}>();

const anchorRef = ref<InstanceType<typeof HTMLElement>>();
const optionRef = ref<InstanceType<typeof HTMLDivElement>>();

const path = computed(() => {
  // fix the MV! for arrays path": "root\u000bdomain\u000btags\u000btag"
  // we need the _last_ `tag` it needs to be the index (e.g. 0, 1, 2...)
  let path = props.attributeTree.prop?.path ?? "";
  // fix the MV!
  path = path.replaceAll("\u000b", "/");
  return path;
});

type AttrData = { value: string };
const wForm = useWatchedForm<AttrData>();
const attrData = computed<AttrData>(() => {
  return { value: props.attributeTree.attributeValue.value };
});

const valueForm = wForm.newForm(attrData, async ({ value }) => {
  emit("save", path.value, props.attributeTree.id, value.value);
});

// i assume more things than comboboxes have a list of options
type AttrOption = string | number;
const maybeOptions = computed<{
  hasOptions: boolean;
  options: LabelList<AttrOption>;
}>(() => {
  if (props.attributeTree.prop?.kind === "boolean") {
    return {
      hasOptions: true,
      options: [
        { label: "true", value: "true" },
        { label: "false", value: "false" },
        { label: "unset", value: "UNSET" },
      ],
    };
  }

  const kind = props.attributeTree.prop?.widgetKind;
  if (!kind) return { hasOptions: false, options: [] };

  // FUTURE: secrets have options
  if (kind instanceof Object) {
    let options: LabelList<AttrOption> | undefined = [];
    if ("comboBox" in kind)
      options = (kind.comboBox as PropertyEditorPropWidgetKindComboBox).options;
    else if ("select" in kind)
      options = (kind.select as PropertyEditorPropWidgetKindSelect).options;

    if (!options) options = [];

    return { hasOptions: true, options };
  }
  return { hasOptions: false, options: [] };
});

const showOptions = ref(false);
const filterFocus = ref(false);

attributeEmitter.on("selectedPath", (selectedPath) => {
  if (selectedPath !== path.value) {
    hideOptions();
  }
});

const hideOptions = () => {
  showOptions.value = false;
};

const triggerCloseCheck = () => {
  // blur fires before focus
  // wait to see if the filter is focused
  setTimeout(() => {
    if (filterFocus.value === false) showOptions.value = false;
  }, 100);
};

const blurFilter = () => {
  filterFocus.value = false;
  triggerCloseCheck();
};

const focus = () => {
  attributeEmitter.emit("selectedPath", path.value);
  showOptions.value = true;
};

const calcOptionPos = () => {
  if (!anchorRef.value) return { top: 0, left: 0 };
  const rect = anchorRef.value.getBoundingClientRect();
  const top = rect.bottom;
  const left = rect.left;
  const pos = {
    top: `${top}px`,
    left: `${left}px`,
  };
  return pos;
};

const optionPos = ref();

onMounted(() => {
  optionPos.value = calcOptionPos();
});

attributeEmitter.on("scrolled", () => {
  if (showOptions.value) {
    optionPos.value = calcOptionPos();
  }
});

const select = (option: LabelEntry<AttrOption>) => {
  valueForm.fieldInfo.value.instance?.handleChange(option.label);
  valueForm.handleSubmit();
  showOptions.value = false;
};

const blur = () => {
  if (valueForm.fieldInfo.value.instance?.state.meta.isDirty) {
    // don't double submit if you were `select()'d'`
    if (!valueForm.baseStore.state.isSubmitted) valueForm.handleSubmit();
    showOptions.value = false;
  } else {
    triggerCloseCheck();
  }
};

const emit = defineEmits<{
  (e: "save", path: string, id: string, value: string): void;
}>();
</script>
