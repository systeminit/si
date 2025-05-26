<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <label
    ref="anchorRef"
    class="grid grid-cols-2 items-center gap-xs relative text-sm font-normal"
  >
    <div class="flex flex-row items-center gap-2xs pl-xs">
      <TruncateWithTooltip>{{ displayName }}</TruncateWithTooltip>
      <IconButton
        v-if="canDelete"
        icon="trash"
        iconTone="destructive"
        iconIdleTone="shade"
        size="sm"
        class="ml-auto"
        loadingIcon="loader"
        :loading="bifrostingTrash"
        @click.left="remove"
      />
    </div>
    <div
      ref="inputFocusDivRef"
      :class="
        clsx(
          'block w-full h-lg p-xs ml-auto text-sm border cursor-text',
          themeClasses(
            'text-shade-100 bg-shade-0 border-neutral-400',
            'text-shade-0 bg-shade-100 border-neutral-600',
          ),
        )
      "
      tabindex="0"
      @focus="openInput"
      @click.left="openInput"
    >
      <TruncateWithTooltip>
        {{
          maybeOptions.options?.find((o) => o.value === attrData.value)
            ?.label ?? attrData.value
        }}
      </TruncateWithTooltip>
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

  <!-- floating input window, shows when this attribute is selected -->
  <template v-if="inputOpen">
    <valueForm.Field name="value">
      <template #default="{ field }">
        <div
          ref="inputWindowRef"
          :class="
            clsx(
              'absolute flex flex-col gap-xs text-sm font-normal border z-100 p-xs',
              themeClasses(
                'bg-shade-0 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
          :style="inputWindowStyles"
        >
          <!-- top input row, looks mostly the same as the unselected input -->
          <div class="grid grid-cols-2 pl-xs gap-2xs">
            <div class="flex flex-row items-center gap-2xs">
              <TruncateWithTooltip>{{ displayName }}</TruncateWithTooltip>
              <IconButton
                v-if="canDelete"
                icon="trash"
                iconTone="destructive"
                iconIdleTone="shade"
                size="sm"
                class="ml-auto"
                loadingIcon="loader"
                :loading="bifrostingTrash"
                @click.left="remove"
              />
            </div>
            <input
              ref="inputRef"
              :class="
                clsx(
                  'block w-full h-lg p-xs ml-auto text-sm border',
                  themeClasses(
                    'text-shade-100 bg-shade-0 border-neutral-400',
                    'text-shade-0 bg-shade-100 border-neutral-600',
                  ),
                )
              "
              type="text"
              :value="field.state.value"
              :disabled="wForm.bifrosting.value || bifrostingTrash"
              @input="(e) => onInputChange(e)"
              @blur="blur"
              @focus="focus"
              @keydown.esc.stop.prevent="closeInput"
              @keydown.up.prevent="onUp"
              @keydown.down.prevent="onDown"
              @keydown.enter.prevent="onEnter"
              @keydown.tab="onTab"
            />
            <TextPill class="absolute text-xs right-sm top-sm">Tab</TextPill>
          </div>

          <!-- raw value selection area -->
          <div
            :class="
              clsx(
                'flex flex-row px-xs justify-between',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            <div>Enter value</div>
            <div
              :class="
                clsx(
                  'text-xs',
                  themeClasses('text-neutral-900', 'text-neutral-200'),
                )
              "
            >
              Navigate
              <span><TextPill>Up</TextPill> <TextPill>Down</TextPill></span>
            </div>
          </div>
          <div
            :class="
              clsx(
                'flex flex-row items-center cursor-pointer border border-transparent',
                'px-xs py-2xs h-[30px]',
                themeClasses(
                  'hover:border-action-500',
                  'hover:border-action-300',
                ),
                selectedIndex === 0 &&
                  themeClasses('bg-action-200', 'bg-action-900'),
              )
            "
            @click.left="selectDefault"
          >
            <TruncateWithTooltip
              :class="
                clsx(
                  'grow',
                  !field.state.value && [
                    'italic',
                    themeClasses('text-neutral-600', 'text-neutral-400'),
                  ],
                )
              "
            >
              <template v-if="field.state.value">
                "{{ field.state.value }}"
              </template>
              <template v-else> No value </template>
            </TruncateWithTooltip>
            <div
              v-if="selectedIndex === 0"
              :class="
                clsx(
                  'text-xs flex-none',
                  themeClasses('text-neutral-900', 'text-neutral-200'),
                )
              "
            >
              <TextPill>Enter</TextPill>
              to select
            </div>
          </div>

          <!-- select value from options area -->
          <div
            v-if="maybeOptions.hasOptions"
            ref="optionRef"
            class="max-h-[10rem] scrollable"
          >
            <div
              :class="
                clsx(
                  'flex flex-row p-xs justify-between',
                  themeClasses('text-neutral-600', 'text-neutral-400'),
                )
              "
            >
              Select Value
            </div>
            <ol>
              <li
                v-for="(option, index) in filteredOptions"
                :key="option.value"
                :class="
                  clsx(
                    'cursor-pointer px-xs py-2xs border border-transparent first:border-transparent',
                    'flex flex-row items-center',
                    isOptionSelected(index) && [
                      'input-selected-item',
                      themeClasses('bg-action-200', 'bg-action-900'),
                    ],
                    themeClasses(
                      'border-t-neutral-400 hover:border-action-500',
                      'border-t-neutral-600 hover:border-action-300',
                    ),
                  )
                "
                @click.left="() => selectOption(option, index)"
              >
                <TruncateWithTooltip class="grow">{{
                  option.label
                }}</TruncateWithTooltip>
                <div
                  v-if="isOptionSelected(index)"
                  :class="
                    clsx(
                      'text-xs flex-none',
                      themeClasses('text-neutral-900', 'text-neutral-200'),
                    )
                  "
                >
                  <TextPill>Enter</TextPill>
                  to select
                </div>
              </li>
              <li v-if="filteredOptions.length === 0" class="p-xs">
                <em>No options found</em>
              </li>
            </ol>
          </div>

          <!-- select potential connection area -->
          <div
            :class="
              clsx(
                'px-xs',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            Or connect to an existing prop
          </div>
          <div class="max-h-[10rem] scrollable">
            <div
              v-for="(connection, index) in filteredConnections"
              :key="connection.attributeValueId"
              :class="
                clsx(
                  'possible-connections grid gap-xs cursor-pointer border border-transparent',
                  'px-xs py-2xs h-[30px]',
                  isConnectionSelected(index) && [
                    'input-selected-item',
                    themeClasses('bg-action-200', 'bg-action-900'),
                  ],
                  themeClasses(
                    'hover:border-action-500',
                    'hover:border-action-300',
                  ),
                  false && themeClasses('bg-action-200', 'bg-action-900'),
                )
              "
              @click.left="selectConnection(index)"
            >
              <TruncateWithTooltip>
                {{ connection.componentName }}
              </TruncateWithTooltip>
              <div class="flex flex-row gap-2xs items-center">
                <template
                  v-for="(item, itemIndex) in connection.pathArray"
                  :key="item"
                >
                  <TruncateWithTooltip
                    class="flex-1 max-w-fit"
                    :style="`flex-basis: ${100 / connection.pathArray.length}%`"
                  >
                    {{ item }}
                  </TruncateWithTooltip>
                  <div v-if="itemIndex !== connection.pathArray.length - 1">
                    /
                  </div>
                </template>
              </div>
              <div
                v-if="isConnectionSelected(index)"
                :class="
                  clsx(
                    'text-xs pt-3xs ml-auto',
                    themeClasses('text-neutral-900', 'text-neutral-200'),
                  )
                "
              >
                <TextPill>Enter</TextPill>
                to select
              </div>
              <TruncateWithTooltip v-else>
                <template
                  v-if="
                    connection.annotation === 'array' ||
                    connection.annotation === 'map' ||
                    connection.annotation === 'object' ||
                    connection.annotation === 'json'
                  "
                >
                  {{ connection.annotation }}
                </template>
                <template v-else> {{ connection.value }} </template>
              </TruncateWithTooltip>
            </div>
          </div>
        </div>
      </template>
    </valueForm.Field>
  </template>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import clsx from "clsx";
import {
  Icon,
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useQuery } from "@tanstack/vue-query";
import {
  PropertyEditorPropWidgetKind,
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import {
  EntityKind,
  PossibleConnection,
  Prop,
} from "@/workers/types/entity_kind_types";
import {
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { attributeEmitter } from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";
import TextPill from "./TextPill.vue";

type UIPotentialConnection = PossibleConnection & {
  pathArray: string[];
};

const props = defineProps<{
  attributeValueId: string;
  path: string;
  value: string;
  kind?: PropertyEditorPropWidgetKind | string;
  prop?: Prop;
  displayName: string;
  canDelete?: boolean;
  disabled?: boolean;
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
const wForm = useWatchedForm<AttrData>(
  `component.av.prop.${props.attributeValueId}`,
);
const attrData = computed<AttrData>(() => {
  return { value: props.value };
});

const valueForm = wForm.newForm({
  data: attrData,
  onSubmit: async ({ value }) => {
    if (connectingComponentId.value) {
      emit(
        "save",
        path.value,
        props.attributeValueId,
        value.value,
        connectingComponentId.value,
      );
    } else {
      emit("save", path.value, props.attributeValueId, value.value);
    }
  },
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

const filterStr = ref("");
const filteredOptions = reactive<LabelList<AttrOption>>([]);
const resetFilteredOptions = () =>
  filteredOptions.splice(0, Infinity, ...maybeOptions.value.options);

watch(
  () => filterStr.value,
  () => {
    if (!filterStr.value) {
      resetFilteredOptions();
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

const selectConnection = (index: number) => {
  if (isConnectionSelected(index)) {
    const newConnection = filteredConnections.value[index];
    if (!newConnection) return;
    const newValue = newConnection.path;
    connectingComponentId.value = newConnection.componentId;
    if (
      newValue &&
      connectingComponentId.value &&
      newValue !== attrData.value.value
    ) {
      valueForm.setFieldValue("value", newValue);
      valueForm.handleSubmit();
    }
    closeInput();
  } else {
    selectedIndex.value = index + 1 + filteredOptions.length;
  }
};
const selectOption = (option: LabelEntry<AttrOption>, index: number) => {
  if (isOptionSelected(index)) {
    const newValue = option.value.toString();
    connectingComponentId.value = undefined;
    if (newValue !== attrData.value.value) {
      valueForm.setFieldValue("value", newValue);
      valueForm.handleSubmit();
    }
    closeInput();
  } else {
    selectedIndex.value = index + 1;
  }
};
const selectDefault = () => {
  if (selectedIndex.value === 0) {
    const newValue = valueForm.state.values.value;
    connectingComponentId.value = undefined;
    if (newValue !== attrData.value.value) {
      valueForm.handleSubmit();
    }
    closeInput();
  } else {
    selectedIndex.value = 0;
  }
};

const blur = () => {
  inputRef.value?.focus();
};

const bifrostingTrash = ref(false);
const remove = () => {
  emit("delete", path.value, props.attributeValueId);
  bifrostingTrash.value = true;
};

// TODO add spinner for deletion
const emit = defineEmits<{
  (
    e: "save",
    path: string,
    id: string,
    value: string,
    connectingComponentId?: string,
  ): void;
  (e: "delete", path: string, id: string): void;
}>();

// INPUT WINDOW LOGIC

const selectedIndex = ref(0);
const inputRef = ref<InstanceType<typeof HTMLInputElement>>();
const inputWindowRef = ref<InstanceType<typeof HTMLDivElement>>();
const inputOpen = ref(false);
const labelRect = ref<undefined | DOMRect>(undefined);

const openInput = () => {
  resetFilteredOptions();
  valueForm.reset();
  labelRect.value = anchorRef.value?.getClientRects()[0];
  if (!labelRect.value) return;
  inputOpen.value = true;
  selectedIndex.value = 0;
  connectingComponentId.value = undefined;
  nextTick(() => {
    inputRef.value?.focus();
    document.addEventListener("mousedown", onClick);
  });
};
const inputWindowStyles = computed(() => {
  // These values account for the padding to get the position right
  const PADDING_AND_BORDER_OFFSET = 10;
  const WIDTH_OFFSET = 16;

  return `width: ${
    (labelRect.value?.width ?? -WIDTH_OFFSET) + WIDTH_OFFSET
  }px; top: ${
    (labelRect.value?.top ?? PADDING_AND_BORDER_OFFSET) -
    PADDING_AND_BORDER_OFFSET
  }px; left: ${
    (labelRect.value?.left ?? PADDING_AND_BORDER_OFFSET) -
    PADDING_AND_BORDER_OFFSET
  }px`;
});
const closeInput = () => {
  inputOpen.value = false;
  document.removeEventListener("mousedown", onClick);
};

const onInputChange = (e: Event) => {
  const v = (e.target as HTMLInputElement).value;
  valueForm.setFieldValue("value", v);
  filterStr.value = v;
  selectedIndex.value = 0;
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
const onUp = () => {
  selectedIndex.value--;
  if (selectedIndex.value < 0) {
    selectedIndex.value =
      filteredConnections.value.length + filteredOptions.length;
  }
};
const onDown = () => {
  selectedIndex.value++;
  if (
    selectedIndex.value >
    filteredConnections.value.length + filteredOptions.length
  ) {
    selectedIndex.value = 0;
  }
};
const onEnter = () => {
  if (selectedIndex.value === 0) {
    selectDefault();
  } else if (selectedIndex.value < filteredOptions.length + 1) {
    const option = filteredOptions[selectedIndex.value - 1];
    if (option) {
      selectOption(option, selectedIndex.value - 1);
    }
  } else {
    selectConnection(selectedIndex.value - filteredOptions.length - 1);
  }
};
const inputFocusDivRef = ref<HTMLDivElement>();
const onTab = (e: KeyboardEvent) => {
  // This allows the user to Tab or Shift+Tab to go through the attribute fields
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  const currentFocus = inputFocusDivRef.value;
  if (!currentFocus) return;
  const index = focusable.indexOf(currentFocus);
  if (e.shiftKey) {
    e.preventDefault();
    closeInput();
    nextTick(() => {
      if (currentFocus && focusable) {
        if (index > 0) {
          focusable[index - 1]?.focus();
        } else {
          focusable[focusable.length - 1]?.focus();
        }
      }
    });
  } else if (index === focusable.length - 1) {
    // When you hit the last attribute, go back to the
    // fuzzy search instead of searching the document for more things to tab to.
    e.preventDefault();
    closeInput();
    nextTick(() => {
      focusable[0]?.focus();
    });
  } else {
    closeInput();
  }
};

const connectingComponentId = ref<string | undefined>();
const annotation = ref<string | null>(null);
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();
const enabled = computed(() => !!annotation.value);
const potentialConnQuery = useQuery({
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
const filteredConnections = computed(() => {
  const output: UIPotentialConnection[] = [];

  if (potentialConnQuery.data.value?.typeMatches) {
    const addToOutput = (matches: PossibleConnection[]) => {
      const fuzzyMatches: PossibleConnection[] = [];

      if (filterStr.value) {
        const fzf = new Fzf(matches, {
          casing: "case-insensitive",
          selector: (match) =>
            `${match.name} ${match.value} ${match.componentName} ${match.path} ${match.schemaName}`,
        });

        const results = fzf.find(filterStr.value);
        const items = results.map((fz) => fz.item);
        fuzzyMatches.push(...items);
      } else {
        fuzzyMatches.push(...matches);
      }

      fuzzyMatches.forEach((match) => {
        const pathArray = match.path.split("/");
        pathArray.shift();
        output.push({
          ...match,
          pathArray,
        });
      });
    };

    addToOutput(potentialConnQuery.data.value.exactMatches);
    addToOutput(potentialConnQuery.data.value.typeMatches);
    addToOutput(potentialConnQuery.data.value.nonMatches);
  }

  return output;
});

watch(
  () => selectedIndex.value,
  () => {
    nextTick(() => {
      const el = document.getElementsByClassName("input-selected-item")[0];
      if (el) {
        el.scrollIntoView({ block: "nearest" });
      }
    });
  },
);

const isOptionSelected = (index: number) =>
  selectedIndex.value > 0 && selectedIndex.value - 1 === index;
const isConnectionSelected = (index: number) =>
  selectedIndex.value > filteredOptions.length - 1 &&
  selectedIndex.value - filteredOptions.length - 1 === index;
</script>

<style lang="css" scoped>
.possible-connections.grid {
  grid-template-columns: minmax(0, 20%) minmax(0, 60%) minmax(0, 20%);
}
</style>
