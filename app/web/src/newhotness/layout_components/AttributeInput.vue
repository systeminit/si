<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <label
    ref="anchorRef"
    class="pl-xs flex flex-row items-center relative text-sm font-normal"
  >
    <div class="flex flex-row items-center gap-2xs">
      <span>{{ displayName }}</span>
      <IconButton
        v-if="canDelete"
        icon="trash"
        iconTone="destructive"
        iconIdleTone="shade"
        size="sm"
        class="ml-auto"
        loadingIcon="loader"
        :loading="bifrostingTrash"
        @click="remove"
      />
    </div>
    <div
      :class="
        clsx(
          'block w-80 h-lg p-xs ml-auto text-sm border cursor-text',
          themeClasses(
            'text-shade-100 bg-shade-0 border-neutral-400',
            'text-shade-0 bg-shade-100 border-neutral-600',
          ),
        )
      "
      @click="openInput"
    >
      {{
        maybeOptions.hasOptions
          ? maybeOptions.options.find((o) => o.value === attrData.value)?.label
          : attrData.value
      }}
    </div>
    <!-- `relative` on label just to "float" this loader above the form input -->
    <Icon
      v-if="wForm.bifrosting.value"
      class="absolute right-2xs"
      name="loader"
      size="sm"
      tone="action"
    />
  </label>

  <Teleport v-if="inputOpen" to="body">
    <valueForm.Field name="value">
      <template #default="{ field }">
        <div
          ref="inputWindowRef"
          :class="
            clsx(
              'absolute flex flex-col gap-xs text-sm font-normal border z-100',
              themeClasses(
                'bg-shade-0 border-neutral-200',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
          :style="inputWindowStyles"
        >
          <div class="flex flex-row pl-xs">
            <div class="flex flex-row items-center gap-2xs">
              <span>{{ displayName }}</span>
              <IconButton
                v-if="canDelete"
                icon="trash"
                iconTone="destructive"
                iconIdleTone="shade"
                size="sm"
                class="ml-auto"
                loadingIcon="loader"
                :loading="bifrostingTrash"
                @click="remove"
              />
            </div>
            <input
              ref="inputRef"
              class="block w-80 ml-auto text-white bg-black border border-neutral-600 disabled:bg-neutral-900 text-sm"
              type="text"
              :value="field.state.value"
              :disabled="wForm.bifrosting.value || bifrostingTrash"
              @input="(e) => onInputChange(e, field)"
              @blur="blur"
              @focus="focus"
              @keydown.esc.stop.prevent="closeInput"
              @keydown.up.prevent="onUp"
              @keydown.down.prevent="onDown"
            />
            <TextPill class="absolute text-xs right-xs top-xs">Tab</TextPill>
          </div>

          <div
            :class="
              clsx(
                'flex flex-row px-xs justify-between',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            <div>Enter value</div>
            <div>
              Navigate
              <span
                :class="
                  clsx(
                    'text-xs',
                    themeClasses('text-neutral-900', 'text-neutral-200'),
                  )
                "
                ><TextPill>Up</TextPill> <TextPill>Down</TextPill></span
              >
            </div>
          </div>
          <div class="p-xs">
            {{
              field.state.value ? `"${field.state.value}"` : "No current value"
            }}
          </div>

          <div
            v-if="maybeOptions.hasOptions"
            ref="optionRef"
            class="max-h-[10rem] scrollable"
          >
            <div
              :class="
                clsx(
                  'flex flex-row px-xs justify-between',
                  themeClasses('text-neutral-600', 'text-neutral-400'),
                )
              "
            >
              Select Value
            </div>
            <ol>
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

          <!-- <div
            :class="
              clsx(
                'px-xs',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            Or connect to an existing prop
          </div>
          <div class="px-xs">
            LIST OF PROP CONNECTIONS GOES HERE
          </div> -->
        </div>
      </template>
    </valueForm.Field>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import clsx from "clsx";
import { Icon, IconButton, themeClasses } from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useQuery } from "@tanstack/vue-query";
import {
  PropertyEditorPropWidgetKind,
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import { EntityKind, Prop } from "@/workers/types/dbinterface";
import {
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { attributeEmitter } from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";
import TextPill from "./TextPill.vue";

const annotation = ref<string | null>(null);
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();
const enabled = computed(() => !!annotation.value);
const _potentialConnQuery = useQuery({
  queryKey: makeKey(EntityKind.PossibleConnections),
  enabled,
  queryFn: async () => {
    if (annotation.value) {
      const conns = await getPossibleConnections({
        ...makeArgs(EntityKind.PossibleConnections),
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
// const optionRef = ref<InstanceType<typeof HTMLDivElement>>();

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

const valueForm = wForm.newForm(
  "component.av.prop",
  attrData,
  async ({ value }) => {
    emit("save", path.value, props.attributeValueId, value.value);
  },
);

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
    closeInput();
  }
});

const focus = () => {
  attributeEmitter.emit("selectedPath", path.value);
  attributeEmitter.emit("selectedDocs", {
    link: props.prop?.docLink ?? "",
    docs: props.prop?.documentation ?? "",
  });
  inputOpen.value = true;
  annotation.value = props.prop?.kind ?? null;
};

const select = (option: LabelEntry<AttrOption>) => {
  valueForm.fieldInfo.value.instance?.handleChange(option.value);
  valueForm.handleSubmit();
  closeInput();
};

const blur = () => {
  inputRef.value?.focus();
};

// const submit = () => {
//   // This make the link/interacting with the docs impossible
//   // attributeEmitter.emit("selectedDocs", null)
//   if (valueForm.fieldInfo.value.instance?.state.meta.isDirty) {
//     // don't double submit if you were `select()'d'`
//     if (!valueForm.baseStore.state.isSubmitted) valueForm.handleSubmit();
//     inputOpen.value = false;
//   } else {
//     inputOpen.value = false;
//   }
// };

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

const inputRef = ref<InstanceType<typeof HTMLInputElement>>();
const inputWindowRef = ref<InstanceType<typeof HTMLDivElement>>();
const inputOpen = ref(false);
const labelRect = ref<undefined | DOMRect>(undefined);
const openInput = () => {
  labelRect.value = anchorRef.value?.getClientRects()[0];
  if (!labelRect.value) return;
  inputOpen.value = true;
  nextTick(() => {
    inputRef.value?.focus();
    document.addEventListener("mousedown", onClick);
  });
};
const inputWindowStyles = computed(
  () =>
    `width: ${labelRect.value?.width || 0}px; top: ${
      labelRect.value?.top
    }px; left: ${labelRect.value?.left}px`,
);
const closeInput = () => {
  inputOpen.value = false;
  document.removeEventListener("mousedown", onClick);
};

// TODO(Wendy) - fix this type!
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const onInputChange = (e: Event, field: any) => {
  const v = (e.target as HTMLInputElement).value;
  field.handleChange(v);
  filterStr.value = v;
};
const onClick = (e: MouseEvent) => {
  const target = e.target;
  if (!(target instanceof Element)) {
    return;
  }
  if (!inputWindowRef.value?.contains(target)) {
    closeInput();
  }
};
const onUp = () => {};
const onDown = () => {};
</script>
