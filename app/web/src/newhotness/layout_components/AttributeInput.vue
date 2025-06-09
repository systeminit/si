<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <label
    ref="anchorRef"
    :class="
      clsx(
        'grid grid-cols-2 items-center gap-xs relative text-sm font-normal',
        inputOpen && 'hidden',
        isSecret && 'mb-[-1px]',
      )
    "
  >
    <div class="flex flex-row items-center gap-2xs pl-xs">
      <TruncateWithTooltip>{{ displayName }}</TruncateWithTooltip>
      <div class="flex flex-row items-cetner ml-auto gap-2xs">
        <IconButton
          v-if="canDelete"
          icon="trash"
          iconTone="destructive"
          iconIdleTone="shade"
          size="sm"
          loadingIcon="loader"
          :loading="bifrostingTrash"
          @click.left="remove"
        />
      </div>
    </div>
    <div
      ref="inputFocusDivRef"
      :class="
        clsx(
          isArray || isMap
            ? [
                'flex flex-row items-center',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              ]
            : themeClasses('text-shade-100', 'text-shade-0'),
          'w-full h-lg p-xs ml-auto text-sm border font-mono cursor-text flex flex-row items-center',
          themeClasses(
            'bg-shade-0 border-neutral-400',
            'bg-shade-100 border-neutral-600',
          ),
        )
      "
      tabindex="0"
      @focus="openInput"
      @click.left="openInput"
    >
      <TruncateWithTooltip>
        <template v-if="isArray"> Set manually or connect to a prop </template>
        <template v-else-if="isMap"> Enter a key </template>
        <AttributeValueBox
          v-else-if="isSetByConnection && props.externalSources"
        >
          <template v-if="isSecret">
            <!-- TODO: Paul make this an actual tailwind color! -->
            <div class="text-[#B2DFB9]">
              {{ props.externalSources[0]?.componentName }} /
              {{ attrData.value }}
            </div>
          </template>
          <template v-else>
            <!-- TODO: Paul make this an actual tailwind color! -->
            <div class="text-[#D4B4FE]">
              {{ props.externalSources[0]?.componentName }}
            </div>
            <div :class="themeClasses('text-neutral-600', 'text-neutral-400')">
              {{ attrData.value }}
            </div>
          </template>
        </AttributeValueBox>
        <!-- TODO(Wendy) make this an actual tailwind color! -->
        <AttributeValueBox
          v-else-if="isSecret && attrData.value"
          class="text-[#B2DFB9]"
        >
          {{ attrData.value }}
        </AttributeValueBox>
        <template v-else>
          {{
            maybeOptions.options?.find((o) => o.value === attrData.value)
              ?.label ?? attrData.value
          }}
        </template>
      </TruncateWithTooltip>
      <div class="ml-auto" />
      <!-- This pushes all the icons to the right side! -->
      <Icon v-if="isArray" name="chevron--down" />
      <Icon
        v-if="wForm.bifrosting.value"
        name="loader"
        size="sm"
        tone="action"
      />
      <!-- NOTE(nick): you need "click.stop" here to prevent the outer click -->
      <Icon
        v-if="props.externalSources && props.externalSources.length > 0"
        v-tooltip="
          props.isSecret
            ? 'Remove subscription to Secret'
            : 'Remove subscription'
        "
        name="x"
        size="sm"
        :class="
          clsx(
            'cursor-pointer hover:scale-110 active:scale-100 bg-neutral-800',
            themeClasses(
              'text-neutral-600 hover:text-shade-100',
              'text-neutral-400 hover:text-shade-0',
            ),
          )
        "
        tabindex="-1"
        @click.stop="removeSubscription"
      />
    </div>
    <!-- `relative` on label just to "float" this loader above the form input -->
  </label>

  <!-- floating input window, shows when this attribute is selected -->
  <template v-if="inputOpen">
    <valueForm.Field name="value">
      <template #default="{ field }">
        <div
          ref="inputWindowRef"
          :class="
            clsx(
              // TODO(Wendy) - for floating version, use absolute!
              'flex flex-col gap-xs text-sm font-normal border z-100 p-xs',
              themeClasses(
                'bg-shade-0 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
          :style="inputWindowStyles"
        >
          <!-- top input row, looks mostly the same as the unselected input -->
          <div class="grid grid-cols-2 pl-xs gap-2xs relative">
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
                  'block w-full h-lg p-xs ml-auto text-sm border font-mono',
                  themeClasses(
                    'text-shade-100 bg-shade-0 border-neutral-400',
                    'text-shade-0 bg-shade-100 border-neutral-600',
                  ),
                )
              "
              type="text"
              :value="isMap ? mapKey : field.state.value"
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
            <Icon
              v-if="wForm.bifrosting.value"
              class="absolute right-[42px] top-xs pointer-events-none"
              name="loader"
              size="sm"
              tone="action"
            />
            <TextPill
              class="absolute text-xs right-xs top-[7px] pointer-events-none"
              >Tab</TextPill
            >
          </div>

          <!-- raw value selection area -->
          <div
            :class="
              clsx(
                'flex flex-row px-xs justify-between font-bold',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            <div>
              <template v-if="isArray"> Add an array item </template>
              <template v-else-if="isMap"> Enter a key </template>
              <template v-else-if="isSecret"> Select a secret </template>
              <template v-else> Enter a value </template>
            </div>
            <div
              v-if="!isMap"
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
            v-if="!isSecret"
            :class="
              clsx(
                'flex flex-row items-center cursor-pointer border border-transparent',
                'px-xs py-2xs h-[30px]',
                themeClasses(
                  'hover:border-action-500',
                  'hover:border-action-300',
                ),
                selectedIndex === 0 && [
                  mapKeyError
                    ? themeClasses('bg-destructive-200', 'bg-destructive-900')
                    : themeClasses('bg-action-200', 'bg-action-900'),
                ],
              )
            "
            @click.left="selectDefault"
          >
            <TruncateWithTooltip
              :class="
                clsx(
                  'grow',
                  !field.state.value &&
                    !isArray && [
                      'italic',
                      themeClasses('text-neutral-600', 'text-neutral-400'),
                    ],
                )
              "
            >
              <template v-if="isArray"> + Set an array item manually </template>
              <template v-else-if="isMap && !mapKey">
                You must enter a key
              </template>
              <template v-else-if="isMap && mapKey"> "{{ mapKey }}" </template>
              <template v-else-if="field.state.value">
                "{{ field.state.value }}"
              </template>
              <template v-else> Set to no value</template>
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
            :class="
              clsx(
                'flex flex-row px-xs justify-between font-bold',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            Or select a value
          </div>
          <div
            v-if="maybeOptions.hasOptions"
            ref="optionRef"
            :class="
              clsx(
                'scrollable',
                selectedIndex < filteredOptions.length + 1
                  ? 'max-h-[10rem]'
                  : 'hidden',
              )
            "
          >
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
          <template v-if="!isMap">
            <div
              v-if="!isSecret"
              :class="
                clsx(
                  'px-xs font-bold',
                  themeClasses('text-neutral-600', 'text-neutral-400'),
                )
              "
            >
              Or connect to an existing prop
            </div>
            <div
              :class="
                clsx(
                  'scrollable',
                  selectedIndex > filteredOptions.length || selectedIndex === 0
                    ? 'max-h-[10rem]'
                    : 'hidden',
                )
              "
            >
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
                      :style="`flex-basis: ${
                        100 / connection.pathArray.length
                      }%`"
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
          </template>

          <!-- display potential connection value area -->
          <div v-if="selectedConnection?.value" class="relative">
            <!--- TODO(Wendy) - this doesn't look right? -->
            <CodeViewer
              :code="JSON.stringify(selectedConnection?.value)"
              showTitle
              :title="selectedConnection.path"
            />
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
  PropertyEditorPropWidgetKindSecret,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import {
  EntityKind,
  ExternalSource,
  PossibleConnection,
  Prop,
} from "@/workers/types/entity_kind_types";
import {
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import CodeViewer from "@/components/CodeViewer.vue";
import { PropKind } from "@/api/sdf/dal/prop";
import { attributeEmitter } from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";
import TextPill from "./TextPill.vue";
import AttributeValueBox from "../AttributeValueBox.vue";

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
  externalSources?: ExternalSource[];
  isArray?: boolean;
  isMap?: boolean;
  isSecret?: boolean;
  disableInputWindow?: boolean;
}>();

const isSetByConnection = computed(
  () => props.externalSources && props.externalSources.length > 0,
);

// TODO(Wendy) - come back to this code when we wanna make the input float again
// const context = inject<ComponentPageContext>("ComponentPageContext");

// const closeOnResizeOrScroll = (e: Event) => {
//   if (inputWindowRef.value && e.target instanceof Node && inputWindowRef.value.contains(e.target)) {
//     // ignore events on descendants
//     return;
//   }
//   if (scrollingToFixPosition.value) {
//     scrollingToFixPosition.value = false;
//   } else if (inputOpen.value) {
//     closeInput();
//   }
// };

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
    if (!props.prop) return;
    if (connectingComponentId.value) {
      emit(
        "save",
        path.value,
        props.attributeValueId,
        value.value,
        props.prop.kind,
        connectingComponentId.value,
      );
    } else {
      emit(
        "save",
        path.value,
        props.attributeValueId,
        value.value,
        props.prop.kind,
      );
    }
  },
  watchFn: () => {
    return [attrData.value, props.externalSources];
  },
});

// i assume more things than comboboxes have a list of options
type AttrOption = string | number;
const secretKind = computed(() => {
  if (
    !props.kind ||
    !(props.kind instanceof Object) ||
    !("secret" in props.kind)
  ) {
    return undefined;
  }

  const options = (props.kind.secret as PropertyEditorPropWidgetKindSecret)
    .options;

  const kindOpt = options.find((opt) => opt.label === "secretKind");

  return kindOpt?.value;
});
const maybeOptions = computed<{
  hasOptions: boolean;
  options: LabelList<AttrOption>;
}>(() => {
  if (!props.kind || props.isSecret) return { hasOptions: false, options: [] };

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

  // Even though secrets have options, they are only used to transfer the secret kind, which is extracted to its own variable (secretKind0
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

    if (props.isArray) {
      emit("add");
    } else if (props.isMap) {
      if (!mapKey.value) {
        mapKeyError.value = true;
        return;
      }
      emit("add", mapKey.value);
    } else if (newValue !== attrData.value.value) {
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
const removeSubscription = () => {
  emit("removeSubscription", path.value, props.attributeValueId);
};

// TODO add spinner for deletion
const emit = defineEmits<{
  (
    e: "save",
    path: string,
    id: string,
    value: string,
    propKind: PropKind,
    connectingComponentId?: string,
  ): void;
  (e: "delete", path: string, id: string): void;
  (e: "removeSubscription", path: string, id: string): void;
  (e: "add", key?: string): void;
  (e: "selected"): void;
}>();

// INPUT WINDOW LOGIC

const mapKey = ref("");
const mapKeyError = ref(false);
const defaultSelectedIndex = () => (props.isSecret ? 1 : 0);
const selectedIndex = ref(defaultSelectedIndex());
const inputRef = ref<InstanceType<typeof HTMLInputElement>>();
const inputWindowRef = ref<InstanceType<typeof HTMLDivElement>>();
const inputOpen = ref(false);
const labelRect = ref<undefined | DOMRect>(undefined);

const openInput = () => {
  if (props.disableInputWindow) {
    emit("selected");
    return;
  }

  resetFilteredOptions();
  valueForm.reset();
  labelRect.value = anchorRef.value?.getClientRects()[0];
  inputWindowYPositionOffset.value = 0;
  mapKey.value = "";
  mapKeyError.value = false;
  if (!labelRect.value) return;
  inputOpen.value = true;
  selectedIndex.value = defaultSelectedIndex();
  connectingComponentId.value = undefined;
  nextTick(() => {
    inputRef.value?.focus();
    addListeners();
    // fixWindowPosition();
  });
};
// const scrollingToFixPosition = ref(false);
const inputWindowYPositionOffset = ref(0);
// TODO(Wendy) - come back to this code when we wanna make the input float again
// const fixWindowPosition = () => {
//   // This function fixes the input floating window position if it is off the bottom of the screen

//   // This number determines the minimum distance between the bottom of the input window and the bottom of the screen
//   const WINDOW_EDGE_PADDING = 10;

//   if (inputWindowRef.value) {
//     const rect = inputWindowRef.value.getBoundingClientRect();
//     const edge = window.innerHeight - WINDOW_EDGE_PADDING;
//     if (rect.bottom > edge) {
//       scrollingToFixPosition.value = true; // don't close on scroll for this!
//       // inputWindowYPositionOffset.value = (rect.bottom - window.innerHeight) + WINDOW_EDGE_PADDING;
//       // console.log(`(${rect.bottom} - ${window.innerHeight}) + 5 = ${inputWindowYPositionOffset.value}`);
//       if (context && anchorRef.value) {
//         const anchorTop = anchorRef.value.getBoundingClientRect().top;
//         const inputWindowHeight = rect.height;
//         const scroll = (anchorTop + inputWindowHeight) - edge;
//         console.log(scroll);
//         context.scrollAttributePanel(context.attributePanelScrollY.value + scroll);
//         // context.scrollAttributePanel();
//         // context.attributePanelScrollY
//       }
//       nextTick(() => {
//         labelRect.value = anchorRef.value?.getClientRects()[0];
//       });
//     }
//   }
// };
const inputWindowStyles = computed(() => {
  // These values account for the padding to get the position right
  // const PADDING_AND_BORDER_OFFSET = 10;
  // const WIDTH_OFFSET = 16;

  // return `width: ${
  //   (labelRect.value?.width ?? -WIDTH_OFFSET) + WIDTH_OFFSET
  // }px; top: ${
  //   (labelRect.value?.top ?? PADDING_AND_BORDER_OFFSET) -
  //   PADDING_AND_BORDER_OFFSET // - inputWindowYPositionOffset.value
  // }px; left: ${
  //   (labelRect.value?.left ?? PADDING_AND_BORDER_OFFSET) -
  //   PADDING_AND_BORDER_OFFSET
  // }px`;
  return {};
});
const closeInput = () => {
  inputOpen.value = false;
  removeListeners();
};

const addListeners = () => {
  window.addEventListener("mousedown", onClick);
  // TODO(Wendy) - come back to this code when we wanna make the input float again
  // window.addEventListener("resize", closeOnResizeOrScroll);
  // window.addEventListener("scroll", closeOnResizeOrScroll, true);
};
const removeListeners = () => {
  window.removeEventListener("mousedown", onClick);
  // TODO(Wendy) - come back to this code when we wanna make the input float again
  // window.removeEventListener("resize", closeOnResizeOrScroll);
  // window.addEventListener("scroll", closeOnResizeOrScroll, true);
};

const onInputChange = (e: Event) => {
  const v = (e.target as HTMLInputElement).value;
  if (props.isMap) {
    mapKey.value = v;
    return;
  }

  valueForm.setFieldValue("value", v);
  filterStr.value = v;
  selectedIndex.value = defaultSelectedIndex();
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
  if (props.isMap) return;

  selectedIndex.value--;
  if (selectedIndex.value < defaultSelectedIndex()) {
    selectedIndex.value =
      filteredConnections.value.length + filteredOptions.length;
  }
};
const onDown = () => {
  if (props.isMap) return;

  selectedIndex.value++;
  if (
    selectedIndex.value >
    filteredConnections.value.length + filteredOptions.length
  ) {
    selectedIndex.value = defaultSelectedIndex();
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
      // Node(victor): We know that secret props on secret defining schemas live on /secrets/kind name
      // This MAY match other secret props on random schemas, but we check the types match. Ideally the MVs at some
      // point should tells us what props are the secret props on the secret defining schemas. But this solves
      // our current UI hurdle - only suggesting valid secrets as connection sources for secret props
      if (props.isSecret) {
        matches = matches.filter(
          (m) => secretKind.value && m.path === `/secrets/${secretKind.value}`,
        );
      }

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

    // todo: rethink this for secrets
    if (!props.isSecret) {
      addToOutput(potentialConnQuery.data.value.nonMatches);
    }
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

const selectedConnection = computed(
  () =>
    filteredConnections.value[selectedIndex.value - 1 - filteredOptions.length],
);
// TODO(Wendy) - come back to this code when we wanna make the input float again
// watch(() => inputWindowRef.value?.getBoundingClientRect().height, () => {
//   // This watcher fixes the window position if the height of the input window div changes
//   nextTick(() => {
//     fixWindowPosition();
//   });
// });
// watch(() => filteredConnections.value, () => {
//   nextTick(() => {
//     fixWindowPosition();
//   });
// });
</script>

<style lang="css" scoped>
.possible-connections.grid {
  grid-template-columns: minmax(0, 20%) minmax(0, 60%) minmax(0, 20%);
}
</style>
