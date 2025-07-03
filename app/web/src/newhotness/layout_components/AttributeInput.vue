<template>
  <valueForm.Field name="value">
    <template #default="{ field }">
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
        <!-- Attribute name -->
        <div class="flex flex-row items-center gap-2xs pl-xs">
          <TruncateWithTooltip>{{ displayName }}</TruncateWithTooltip>
          <div class="flex flex-row items-center ml-auto gap-2xs">
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

        <!-- Display / edit the value -->
        <div
          ref="inputFocusDivRef"
          v-tooltip="
            readOnly
              ? {
                  placement: 'left',
                  content: 'Unable to edit this value.',
                }
              : null
          "
          :class="
            clsx(
              'w-full h-lg p-xs ml-auto text-sm border font-mono flex flex-row items-center gap-3xs',
              themeClasses('border-neutral-400', 'border-neutral-600'),

              readOnly
                ? [
                    'cursor-not-allowed',
                    themeClasses(
                      'bg-caution-lines-light text-neutral-600',
                      'bg-caution-lines-dark text-neutral-400',
                    ),
                  ]
                : [themeClasses('bg-shade-0', 'bg-shade-100'), 'cursor-text'],

              isArray || isMap
                ? [
                    'flex flex-row items-center',
                    themeClasses('text-neutral-600', 'text-neutral-400'),
                  ]
                : [!readOnly && themeClasses('text-shade-100', 'text-shade-0')],
            )
          "
          tabindex="0"
          @focus="openInput"
          @click.left="openInput"
        >
          <TruncateWithTooltip>
            <template v-if="isArray">
              Set manually or connect to a prop
            </template>
            <template v-else-if="isMap"> Enter a key </template>
            <AttributeValueBox
              v-else-if="isSetByConnection && props.externalSources"
            >
              <template v-if="isSecret">
                <!-- TODO: Paul make this an actual tailwind color! -->
                <div
                  :class="
                    clsx(
                      'max-w-full flex flex-row items-center [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit',
                      themeClasses(
                        'text-green-light-mode',
                        'text-green-dark-mode',
                      ),
                    )
                  "
                >
                  <TruncateWithTooltip>{{
                    props.externalSources[0]?.componentName
                  }}</TruncateWithTooltip>
                  <div class="flex-none">/</div>
                  <TruncateWithTooltip>
                    {{ field.state.value }}
                  </TruncateWithTooltip>
                </div>
              </template>
              <div
                v-else
                class="max-w-full flex flex-row items-center [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit"
              >
                <!-- TODO: Paul make this an actual tailwind color! -->
                <TruncateWithTooltip class="text-purple">
                  {{ props.externalSources[0]?.componentName }}
                </TruncateWithTooltip>
                <div class="flex-none">/</div>
                <TruncateWithTooltip
                  :class="themeClasses('text-neutral-600', 'text-neutral-400')"
                >
                  {{
                    field.state.value ||
                    `<${props.externalSources[0]?.path?.replace(/^\//, "")}>`
                  }}
                </TruncateWithTooltip>
              </div>
            </AttributeValueBox>
            <!-- TODO(Wendy) make this an actual tailwind color! -->
            <AttributeValueBox
              v-else-if="isSecret && field.state.value"
              :class="
                themeClasses('text-green-light-mode', 'text-green-dark-mode')
              "
            >
              {{ field.state.value }}
            </AttributeValueBox>
            <template v-else>
              {{
                maybeOptions.options?.find((o) => o.value === field.state.value)
                  ?.label ?? field.state.value
              }}
            </template>
          </TruncateWithTooltip>
          <div class="ml-auto" />
          <!-- This pushes all the icons to the right side! -->
          <Icon v-if="isArray" name="chevron--down" />
          <!-- NOTE(nick): you need "click.stop" here to prevent the outer click -->
          <Icon
            v-if="props.externalSources && props.externalSources.length > 0"
            v-tooltip="
              props.isSecret
                ? 'Remove connection to Secret'
                : 'Remove connection'
            "
            name="x"
            size="sm"
            :class="
              clsx(
                'cursor-pointer hover:scale-110 active:scale-100 text-neutral-400',
                themeClasses(
                  'bg-neutral-200 hover:text-shade-100 hover:bg-neutral-300',
                  'bg-neutral-800 hover:text-shade-0 hover:bg-neutral-700',
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
        <div
          ref="inputWindowRef"
          :class="
            clsx(
              'flex flex-col gap-xs text-sm font-normal border z-100 p-xs',
              themeClasses(
                'bg-shade-0 border-neutral-400',
                'bg-neutral-800 border-neutral-600',
              ),
            )
          "
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
            <component
              :is="inputHtmlTag"
              ref="inputRef"
              :class="
                clsx(
                  inputHtmlTag === 'input' && 'h-lg',
                  inputHtmlTag === 'textarea' && 'min-h-[36px]',
                  kindAsString === 'codeeditor' && 'pr-[32px]',
                  'block w-full p-xs ml-auto text-sm border font-mono',
                  'focus:outline-none focus:ring-0 focus:z-10',
                  themeClasses(
                    'text-shade-100 bg-shade-0 border-neutral-400 focus:border-action-500',
                    'text-shade-0 bg-shade-100 border-neutral-600 focus:border-action-300',
                  ),
                )
              "
              :type="inputHtmlTag === 'input' ? 'text' : null"
              :rows="inputHtmlTag === 'textarea' ? 4 : null"
              data-1p-ignore
              :value="isMap ? mapKey : field.state.value"
              :disabled="wForm.bifrosting.value || bifrostingTrash"
              @input="(e: Event) => onInputChange(e)"
              @blur="blur"
              @focus="focus"
              @keydown.esc.stop.prevent="closeInput"
              @keydown.up.prevent="onUp"
              @keydown.down.prevent="onDown"
              @keydown.enter.prevent="onEnter"
              @keydown.tab="onTab"
            />
            <Icon
              v-if="kindAsString === 'codeeditor'"
              v-tooltip="'Set manual value in code editor'"
              name="code-pop"
              size="sm"
              :class="
                clsx(
                  'absolute right-[6px] top-[10px] z-20',
                  themeClasses(
                    'hover:text-action-500',
                    'hover:text-action-300',
                  ),
                  'hover:scale-110 cursor-pointer',
                )
              "
              @click.stop="openCodeEditorModal"
            />
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
              <span
                ><TextPill variant="key2">Up</TextPill>
                <TextPill variant="key2">Down</TextPill></span
              >
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
              <TextPill variant="key2">Enter</TextPill>
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
                  <TextPill variant="key2">Enter</TextPill>
                  to select
                </div>
              </li>
              <li v-if="filteredOptions.length === 0" class="p-xs">
                <em>No options found</em>
              </li>
            </ol>
          </div>

          <!-- select potential connection area -->
          <template v-if="!isMap && filteredConnections.length > 0">
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
            <!--
              Attach the virtualizer to this element. It will use the width and height of
              this element, and will own the scrollbar.

              This will allow us to only create HTML elements for visible items, and speeds up
              the rendering and initialization of the list.
            -->
            <div
              ref="filteredConnectionsListRef"
              :class="
                clsx(
                  'scrollable',
                  selectedIndex > filteredOptions.length || selectedIndex === 0
                    ? 'h-[10rem]'
                    : 'hidden',
                )
              "
            >
              <!-- Create a relative-positioned container so that children are relative to its (0,0) -->
              <div
                :class="clsx('relative w-full')"
                :style="{
                  height: `${filteredConnectionsList.getTotalSize()}px`,
                }"
              >
                <!-- position this item exactly where the virtualizer tells it to go -->
                <div
                  v-for="virtualItem in filteredConnectionsList.getVirtualItems()"
                  :key="
                    filteredConnections[virtualItem.index]?.attributeValueId
                  "
                  :class="
                    clsx(
                      `absolute top-0 left-0 w-full h-[${virtualItem.size}px]`,
                      'possible-connections grid gap-xs cursor-pointer border border-transparent',
                      'px-xs py-2xs',
                      isConnectionSelected(virtualItem.index) && [
                        'input-selected-item',
                        themeClasses('bg-action-200', 'bg-action-900'),
                      ],
                      themeClasses(
                        'hover:border-action-500',
                        'hover:border-action-300',
                      ),
                    )
                  "
                  :style="{
                    transform: `translateY(${virtualItem.start}px)`,
                  }"
                  @click.left="selectConnection(virtualItem.index)"
                >
                  <TruncateWithTooltip>
                    {{ filteredConnections[virtualItem.index]?.componentName }}
                  </TruncateWithTooltip>
                  <div class="flex flex-row gap-2xs items-center">
                    <template
                      v-for="(item, itemIndex) in filteredConnections[
                        virtualItem.index
                      ]?.pathArray"
                      :key="item"
                    >
                      <TruncateWithTooltip
                        class="flex-1 max-w-fit"
                        :style="`flex-basis: ${
                          100 /
                          (filteredConnections[virtualItem.index]?.pathArray
                            .length ?? 0)
                        }%`"
                      >
                        {{ item }}
                      </TruncateWithTooltip>
                      <div
                        v-if="
                          itemIndex !==
                          (filteredConnections[virtualItem.index]?.pathArray
                            .length ?? 0) -
                            1
                        "
                      >
                        /
                      </div>
                    </template>
                  </div>
                  <div
                    v-if="isConnectionSelected(virtualItem.index)"
                    :class="
                      clsx(
                        'text-xs pt-3xs ml-auto',
                        themeClasses('text-neutral-900', 'text-neutral-200'),
                      )
                    "
                  >
                    <TextPill variant="key2">Enter</TextPill>
                    to select
                  </div>
                  <TruncateWithTooltip v-else>
                    <template
                      v-if="
                        filteredConnections[virtualItem.index]?.kind ===
                          'array' ||
                        filteredConnections[virtualItem.index]?.kind ===
                          'map' ||
                        filteredConnections[virtualItem.index]?.kind ===
                          'object' ||
                        filteredConnections[virtualItem.index]?.kind === 'json'
                      "
                    >
                      {{ filteredConnections[virtualItem.index]?.kind }}
                    </template>
                    <template v-else>
                      {{ filteredConnections[virtualItem.index]?.value }}
                    </template>
                  </TruncateWithTooltip>
                </div>
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
      <CodeEditorModal
        ref="codeEditorModalRef"
        :title="`Set Value For ${displayName}`"
        :codeEditorId="`${displayName}-${changeSetId}-${prop?.id}`"
        @submit="setValueFromCodeEditorModal"
      />
    </template>
  </valueForm.Field>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import { debounce } from "lodash-es";
import clsx from "clsx";
import {
  Icon,
  IconButton,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useQuery } from "@tanstack/vue-query";
import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  PropertyEditorPropWidgetKind,
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSecret,
  PropertyEditorPropWidgetKindSelect,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import {
  BifrostComponent,
  EntityKind,
  ExternalSource,
  PossibleConnection,
  Prop,
} from "@/workers/types/entity_kind_types";
import {
  changeSetId,
  getPossibleConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import CodeViewer from "@/components/CodeViewer.vue";
import { PropKind } from "@/api/sdf/dal/prop";
import { CategorizedPossibleConnections } from "@/workers/types/dbinterface";
import {
  attributeEmitter,
  MouseDetails,
  mouseEmitter,
} from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";
import AttributeValueBox from "../AttributeValueBox.vue";
import CodeEditorModal from "../CodeEditorModal.vue";

type UIPotentialConnection = PossibleConnection & {
  pathArray: string[];
};

const props = defineProps<{
  attributeValueId: string;
  path: string;
  value: string;
  kind?: PropertyEditorPropWidgetKind | string;
  prop?: Prop;
  component: BifrostComponent;
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

const anchorRef = ref<InstanceType<typeof HTMLElement>>();

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

const filterStr = ref<string>("");
const filteredOptions = reactive<LabelList<AttrOption>>([]);
const resetFilteredOptions = () =>
  filteredOptions.splice(0, Infinity, ...maybeOptions.value.options);

const debouncedFilterStr = debounce(
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
  500,
  { trailing: true, leading: false },
);

watch(
  () => filterStr.value,
  () => {
    debouncedFilterStr();
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
/**
 * The index of the selected option or connection.
 *
 * This is an index into an imagined concatenated list containing both filteredOptions and filteredConnections.
 *
 * - If this is 0, nothing is selected.
 * - If this is > 0 and <= filteredOptions.length, it references filteredOptions[selectedIndex - 1].
 * - If this is > filteredOptions.length, it references filteredConnetions[selectedIndex - 1 - filteredOptions.length].
 */
const selectedIndex = ref(defaultSelectedIndex());
const inputRef = ref<InstanceType<typeof HTMLInputElement>>();
const inputWindowRef = ref<InstanceType<typeof HTMLDivElement>>();
const inputOpen = ref(false);
const labelRect = ref<undefined | DOMRect>(undefined);

const openInput = () => {
  if (readOnly.value) return;

  if (props.disableInputWindow) {
    emit("selected");
    return;
  }

  resetFilteredOptions();
  valueForm.reset();
  labelRect.value = anchorRef.value?.getClientRects()[0];
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
const closeInput = () => {
  inputOpen.value = false;
  removeListeners();
};

const addListeners = () => {
  mouseEmitter.on("mousedown", onMouseDown);
};
const removeListeners = () => {
  mouseEmitter.off("mousedown", onMouseDown);
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
const onMouseDown = (e: MouseDetails["mousedown"]) => {
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
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();
const queryKey = makeKey(EntityKind.PossibleConnections);
const potentialConnQuery = useQuery({
  queryKey,
  enabled: inputOpen,
  queryFn: async () => {
    if (props.prop) {
      return await getPossibleConnections(
        makeArgs(EntityKind.PossibleConnections),
      );
    }
  },
});

const categorizedPossibleConn = computed(() => {
  const possible = potentialConnQuery.data.value;
  const categories: CategorizedPossibleConnections = {
    suggestedMatches: [],
    typeAndNameMatches: [],
    typeMatches: [],
    nonMatches: [],
  };
  if (!possible) return categories;

  for (const source of possible) {
    const isSuggested =
      props.prop?.suggestSources?.some(
        (s) => s.schema === source.schemaName && s.prop === source.path,
      ) ||
      source.suggestAsSourceFor?.some(
        (d) =>
          d.schema === props.component.schemaName &&
          `root${d.prop}` === props.prop?.path,
      );
    if (isSuggested) {
      categories.suggestedMatches.push(source);
    } else if (
      source.kind === props.prop?.kind ||
      (source.kind === "string" &&
        !["string", "boolean", "object", "map", "integer"].includes(
          props.prop?.kind ?? "",
        ))
    ) {
      // If the types match, sort name matches first
      if (
        source.name === props.prop?.name &&
        source.schemaName !== props.component.schemaName
      ) {
        categories.typeAndNameMatches.push(source);
      } else {
        categories.typeMatches.push(source);
      }
    } else {
      categories.nonMatches.push(source);
    }
  }

  const cmp = (a: PossibleConnection, b: PossibleConnection) =>
    `${a.name} ${a.path}`.localeCompare(`${b.name} ${b.path}`);
  categories.suggestedMatches.sort(cmp);
  categories.typeAndNameMatches.sort(cmp);
  categories.typeMatches.sort(cmp);
  categories.nonMatches.sort(cmp);
  return categories;
});

const filteredConnections = computed(() => {
  const output: UIPotentialConnection[] = [];

  if (potentialConnQuery.data.value) {
    const addToOutput = (matches: PossibleConnection[]) => {
      // Node(victor): We know that secret props on secret defining schemas live on /secrets/kind name
      // This MAY match other secret props on random schemas, but we check the types match. Ideally the MVs at some
      // point should tells us what props are the secret props on the secret defining schemas. But this solves
      // our current UI hurdle - only suggesting valid secrets as connection sources for secret props
      if (props.isSecret) {
        matches = matches.filter(
          (m) => secretKind.value && m.path === `/secrets/${secretKind.value}`,
        );
      } else {
        matches = matches.filter((m) => !m.path.startsWith("/secrets/"));
      }

      matches = matches.filter((m) => m.componentId !== props.component.id);

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

    addToOutput(categorizedPossibleConn.value.suggestedMatches);
    addToOutput(categorizedPossibleConn.value.typeAndNameMatches);
    addToOutput(categorizedPossibleConn.value.typeMatches);

    // todo: rethink this for secrets
    if (!props.isSecret) {
      addToOutput(categorizedPossibleConn.value.nonMatches);
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

// Virtualized list for potential connections
const filteredConnectionsListRef = ref<HTMLDivElement>();
const filteredConnectionsList = useVirtualizer(
  computed(() => {
    return {
      count: filteredConnections.value.length,
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      getScrollElement: () => filteredConnectionsListRef.value!,
      // getItemKey: (index: number) => filteredConnections.value[index]?.attributeValueId ?? "<unknown>",
      estimateSize: () => 30,
      overscan: 3,
    };
  }),
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

const readOnly = computed(
  () => !!(props.prop?.createOnly && props.component.hasResource),
);

const kindAsString = computed(() => `${props.prop?.widgetKind}`.toLowerCase());

const inputHtmlTag = computed(() => {
  if (kindAsString.value === "textarea") {
    return "textarea";
  }

  return "input";
});

const codeEditorModalRef = ref<InstanceType<typeof CodeEditorModal>>();

const openCodeEditorModal = () => {
  const currentValue = valueForm.getFieldValue("value");
  codeEditorModalRef.value?.open(currentValue);
};

const setValueFromCodeEditorModal = (value: string) => {
  valueForm.setFieldValue("value", value);
  valueForm.handleSubmit();
};
</script>

<style lang="css" scoped>
.possible-connections.grid {
  grid-template-columns: minmax(0, 20%) minmax(0, 60%) minmax(0, 20%);
}
</style>
