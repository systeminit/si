<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <label
    ref="anchorRef"
    :class="
      clsx(
        showOptions && 'bg-neutral-500',
        'pl-xs flex flex-row items-center relative',
      )
    "
  >
    <div class="flex flex-row items-center">
      <span>{{ displayName }}</span>
      <IconButton
        v-if="canDelete"
        icon="trash"
        class="ml-auto"
        loadingIcon="loader"
        :loading="bifrostingTrash"
        @click="remove"
      />
    </div>
    <valueForm.Field name="value">
      <template #default="{ field }">
        <input
          class="block w-72 ml-auto text-white bg-black border-2 border-neutral-300 disabled:bg-neutral-900"
          type="text"
          :value="
            maybeOptions.hasOptions
              ? maybeOptions.options.find((o) => o.value === field.state.value)
                  ?.label
              : field.state.value
          "
          :disabled="wForm.bifrosting.value || bifrostingTrash"
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

  <div
    v-show="maybeOptions.hasOptions && showOptions"
    ref="optionRef"
    class="h-[12rem] bg-neutral-500"
  >
    <ol class="scrollable h-full">
      <li class="p-xs">
        <input
          v-model="filterStr"
          type="text"
          class="text-white bg-black border-2 border-neutral-300 w-full block"
          placeholder="filter..."
          @focus="() => (filterFocus = true)"
          @blur="blurFilter"
        />
      </li>
      <li
        v-for="option in filteredOptions"
        :key="option.value"
        class="cursor-pointer p-xs hover:bg-black"
        @mousedown="select(option)"
      >
        {{ option.label }}
      </li>
      <li v-if="filteredOptions.length === 0" class="p-xs">
        <em>No options found</em>
      </li>
    </ol>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import clsx from "clsx";
import { Icon, IconButton } from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useQuery } from "@tanstack/vue-query";
import {
  PropertyEditorPropWidgetKind,
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import { Prop } from "@/workers/types/dbinterface";
import {
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { attributeEmitter } from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";

const annotation = ref<string | null>(null);
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();
const enabled = computed(() => !!annotation.value);
const _potentialConnQuery = useQuery({
  queryKey: makeKey("PossibleConnections"),
  enabled,
  queryFn: async () => {
    if (annotation.value) {
      const conns = await getPossibleConnections({
        ...makeArgs("PossibleConnections"),
        annotation: annotation.value,
      });
      return conns;
    }
  },
});

const props = defineProps<{
  attributeValueId: string;
  path: string;
  value: string;
  kind?: PropertyEditorPropWidgetKind | string;
  prop?: Prop;
  displayName: string;
  canDelete?: boolean;
}>();

const anchorRef = ref<InstanceType<typeof HTMLElement>>();
const optionRef = ref<InstanceType<typeof HTMLDivElement>>();

const path = computed(() => {
  // fix the MV! for arrays path": "root\u000bdomain\u000btags\u000btag"
  let path = props.path;
  path = path.replaceAll("\u000b", "/");
  return path;
});

type AttrData = { value: string };
const wForm = useWatchedForm<AttrData>();
const attrData = computed<AttrData>(() => {
  return { value: props.value };
});

const valueForm = wForm.newForm(attrData, async ({ value }) => {
  emit("save", path.value, props.attributeValueId, value.value);
});

// i assume more things than comboboxes have a list of options
type AttrOption = string | number;
const maybeOptions = computed<{
  hasOptions: boolean;
  options: LabelList<AttrOption>;
}>(() => {
  if (!props.kind) return { hasOptions: false, options: [] };

  if (props.kind === "boolean") {
    return {
      hasOptions: true,
      options: [
        { label: "true", value: "true" },
        { label: "false", value: "false" },
        { label: "unset", value: "UNSET" },
      ],
    };
  }

  // FUTURE: secrets have options
  if (props.kind instanceof Object) {
    let options: LabelList<AttrOption> | undefined = [];
    if ("comboBox" in props.kind)
      options = (props.kind.comboBox as PropertyEditorPropWidgetKindComboBox)
        .options;
    else if ("select" in props.kind)
      options = (props.kind.select as PropertyEditorPropWidgetKindSelect)
        .options;

    if (!options) options = [];

    return { hasOptions: true, options };
  }
  return { hasOptions: false, options: [] };
});

const showOptions = ref(false);
const filterFocus = ref(false);

const filterStr = ref("");
const filteredOptions = reactive<LabelList<AttrOption>>([]);

watch(
  () => filterStr.value,
  () => {
    if (!filterStr.value) {
      filteredOptions.splice(0, Infinity, ...maybeOptions.value.options);
      return;
    }

    const fzf = new Fzf(maybeOptions.value.options, {
      casing: "case-insensitive",
      selector: (o) => `${o.value} ${o.label}`,
    });

    const results = fzf.find(filterStr.value);
    const items: LabelList<AttrOption> = results.map((fz) => fz.item);
    filteredOptions.splice(0, Infinity, ...items);
  },
  { immediate: true },
);

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
  attributeEmitter.emit("selectedDocs", {
    link: props.prop?.docLink ?? "",
    docs: props.prop?.documentation ?? "",
  });
  showOptions.value = true;
  annotation.value = props.prop?.kind ?? null;
};

const select = (option: LabelEntry<AttrOption>) => {
  valueForm.fieldInfo.value.instance?.handleChange(option.value);
  valueForm.handleSubmit();
  showOptions.value = false;
};

const blur = () => {
  // This make the link/interacting with the docs impossible
  // attributeEmitter.emit("selectedDocs", null)
  if (valueForm.fieldInfo.value.instance?.state.meta.isDirty) {
    // don't double submit if you were `select()'d'`
    if (!valueForm.baseStore.state.isSubmitted) valueForm.handleSubmit();
    showOptions.value = false;
  } else {
    triggerCloseCheck();
  }
};

const bifrostingTrash = ref(false);
const remove = () => {
  emit("delete", path.value, props.attributeValueId);
  bifrostingTrash.value = true;
};

// TODO add spinner for deletion
const emit = defineEmits<{
  (e: "save", path: string, id: string, value: string): void;
  (e: "delete", path: string, id: string): void;
}>();
</script>
