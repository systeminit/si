<template>
  <valueForm.Field name="value">
    <template #default="{ field }">
      <!-- eslint-disable vue/no-multiple-template-root -->
      <label
        ref="anchorRef"
        :class="
          clsx(
            'grid grid-cols-2 items-center gap-2xs relative text-sm font-normal',
            inputOpen && 'hidden',
            isSecret && 'mb-[-1px]',
            (hasError || validationStatus === 'failing') && [
              'pr-xs',
              themeClasses('bg-destructive-200', 'bg-newhotness-destructive'),
            ],
            props.hasSocketConnection && 'pr-xs',
          )
        "
        :style="
          props.hasSocketConnection
            ? { backgroundColor: 'rgba(125, 74, 23, 0.25)' }
            : {}
        "
      >
        <!-- Attribute name -->
        <div
          :class="
            clsx(
              'flex flex-row items-center gap-2xs pl-xs',
              (hasError || validationStatus === 'failing') && 'mt-xs',
            )
          "
        >
          <AttributeInputRequiredProperty
            :text="displayName"
            :showAsterisk="validationStatus === 'missingRequiredValue'"
          />
          <div class="flex flex-row items-center ml-auto gap-2xs">
            <NewButton
              v-if="canDelete && !component.toDelete"
              ref="deleteButtonRef"
              tooltip="Delete"
              tooltipPlacement="top"
              icon="trash"
              tone="destructive"
              loadingIcon="loader"
              :loading="bifrostingTrash"
              loadingText=""
              :tabIndex="0"
              :class="
                clsx(
                  'focus:outline',
                  themeClasses(
                    'focus:outline-action-500',
                    'focus:outline-action-300',
                  ),
                )
              "
              @click.left="remove"
              @keydown.tab.stop.prevent="onDeleteButtonTab"
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

              hasError || validationStatus === 'failing'
                ? [
                    'mt-xs',
                    themeClasses(
                      'border-destructive-600',
                      'border-destructive-400',
                    ),
                  ]
                : props.hasSocketConnection
                ? [
                    'mt-xs',
                    themeClasses('border-neutral-400', 'border-neutral-600'),
                  ]
                : themeClasses('border-neutral-400', 'border-neutral-600'),

              readOnly
                ? [
                    'cursor-not-allowed focus:outline-none focus:z-10',
                    themeClasses(
                      'bg-neutral-100 text-neutral-600 focus:border-action-500',
                      'bg-neutral-900 text-neutral-400 focus:border-action-300',
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
          :tabindex="readOnly ? -1 : 0"
          @focus="(e) => !readOnly && openInput()"
          @keydown.tab="(e) => (readOnly ? onTab(e) : null)"
          @click.left="(e) => !readOnly && openInput()"
        >
          <TruncateWithTooltip>
            <template v-if="(isArray || isMap) && !isSetByConnection">
              <!-- arrays and maps do not show a value here! -->
            </template>
            <AttributeValueBox
              v-else-if="isSetByConnection && externalSources"
              :class="clsx(attributeInputContext?.blankInput && 'border-0')"
            >
              <template v-if="isSecret">
                <template v-if="field.state.value">
                  <!-- TODO: Paul make this an actual tailwind color! -->
                  <div
                    :class="
                      clsx(
                        'max-w-full flex flex-row items-center [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit',
                        themeClasses(
                          'text-newhotness-greenlight',
                          'text-newhotness-greendark',
                        ),
                      )
                    "
                  >
                    <TruncateWithTooltip>{{
                      externalSources[0]?.componentName
                    }}</TruncateWithTooltip>
                    <div class="flex-none">/</div>
                    <TruncateWithTooltip>
                      {{ field.state.value }}
                    </TruncateWithTooltip>
                  </div>
                </template>
              </template>
              <div
                v-else-if="externalSources.length > 0"
                class="max-w-full flex flex-row items-center [&>*]:min-w-0 [&>*]:flex-1 [&>*]:max-w-fit"
              >
                <!-- TODO: Paul make this an actual tailwind color! -->
                <TruncateWithTooltip
                  :class="
                    themeClasses(
                      'text-newhotness-purplelight',
                      'text-newhotness-purpledark',
                    )
                  "
                >
                  {{ externalSources[0]?.componentName }}
                </TruncateWithTooltip>
                <div class="flex-none">/</div>
                <TruncateWithTooltip
                  :class="themeClasses('text-neutral-600', 'text-neutral-400')"
                >
                  {{
                    field.state.value ||
                    `<${externalSources[0]?.path?.replace(/^\//, "")}>`
                  }}
                </TruncateWithTooltip>
              </div>
            </AttributeValueBox>
            <!-- TODO(Wendy) make this an actual tailwind color! -->
            <AttributeValueBox
              v-else-if="isSecret && field.state.value"
              :class="
                themeClasses(
                  'text-newhotness-greenlight',
                  'text-newhotness-greendark',
                )
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
          <!-- This pushes all the icons to the right side! -->
          <div class="ml-auto" />
          <!-- NOTE(nick): you need "click.stop" here to prevent the outer click -->
          <Icon
            v-if="
              props.externalSources &&
              props.externalSources.length > 0 &&
              !component.toDelete
            "
            v-tooltip="
              props.isSecret
                ? 'Remove subscription to Secret'
                : 'Remove subscription'
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

      <!-- validation message below the name and input box -->
      <div
        v-if="
          !inputOpen &&
          (hasError ||
            (validationStatus === 'failing' && props.validation?.message))
        "
        :class="
          clsx(
            'flex flex-row p-xs text-sm',
            themeClasses(
              'text-destructive-600 bg-destructive-200',
              'text-destructive-200 bg-newhotness-destructive',
            ),
          )
        "
      >
        <span v-if="props.validation?.message">
          {{ props.validation.message }}
        </span>
        <span v-else-if="hasError"> `{{ errorValue }}` failed to save </span>
      </div>

      <!-- socket connections incompatibility message -->
      <div
        v-if="!inputOpen && props.hasSocketConnection"
        :class="clsx('flex flex-row p-xs text-xs')"
        :style="{ backgroundColor: 'rgba(125, 74, 23, 0.25)' }"
      >
        <span>
          This attribute setting is incompatible with the new experience
        </span>
      </div>

      <!-- floating input window, shows when this attribute is selected -->
      <template v-if="inputOpen">
        <div
          ref="inputWindowRef"
          :class="
            clsx(
              'flex flex-col gap-xs text-sm font-normal border z-100 p-xs',
              themeClasses(
                'bg-neutral-100 border-neutral-400',
                'bg-neutral-700 border-neutral-500',
              ),
            )
          "
        >
          <!-- top input row, looks mostly the same as the unselected input -->
          <div class="grid grid-cols-2 pl-xs gap-2xs relative">
            <div class="flex flex-row items-center gap-2xs">
              <AttributeInputRequiredProperty
                :text="displayName"
                :showAsterisk="validationStatus === 'missingRequiredValue'"
              />
              <NewButton
                v-if="canDelete && !component.toDelete"
                tooltip="Delete (⌘⌫)"
                tooltipPlacement="top"
                icon="trash"
                tone="destructive"
                loadingIcon="loader"
                :loading="bifrostingTrash"
                loadingText=""
                :tabIndex="-1"
                :class="
                  clsx(
                    'ml-auto focus:outline',
                    themeClasses(
                      'focus:outline-action-500',
                      'focus:outline-action-300',
                    ),
                  )
                "
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
              data-lpignore="true"
              data-1p-ignore
              data-bwignore
              data-form-type="other"
              :value="isMap ? mapKey : field.state.value"
              :disabled="wForm.bifrosting.value || bifrostingTrash"
              @input="(e: Event) => onInputChange(e)"
              @blur="blur"
              @focus="focus"
              @keydown.esc.stop.prevent="closeAndReset"
              @keydown.up="onUp"
              @keydown.down="onDown"
              @keydown.enter.prevent="onEnter"
              @keydown.tab="onTab"
              @keydown.delete="onDelete"
            />
            <Icon
              v-if="kindAsString === 'codeeditor'"
              v-tooltip="'Set manual value in code editor'"
              name="code-pop-right"
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

          <!-- error display -->
          <div
            v-if="hasError"
            :class="
              clsx(
                'p-xs text-sm',
                themeClasses(
                  'text-destructive-600 bg-destructive-200',
                  'text-destructive-200 bg-newhotness-destructive',
                ),
              )
            "
          >
            `{{ errorValue }}` failed to save
          </div>

          <!-- raw value selection area -->
          <div
            :class="
              clsx(
                'flex flex-row items-center gap-sm px-xs font-bold h-5',
                themeClasses('text-neutral-600', 'text-neutral-400'),
              )
            "
          >
            <TruncateWithTooltip>
              <template v-if="isArray"> Add an array item manually </template>
              <template v-else-if="isMap"> Enter a key </template>
              <template v-else-if="isSecret"> Select a secret </template>
              <template v-else> Enter a value </template>
            </TruncateWithTooltip>

            <!-- Divides the controls from the text and pushes them right -->
            <div class="ml-auto" />

            <div
              v-if="selectedIndex === -1"
              :class="
                clsx(
                  'text-xs flex-none flex flex-row items-center gap-2xs',
                  themeClasses('text-neutral-900', 'text-neutral-200'),
                )
              "
            >
              <div>
                {{ inputTouched ? discardString : "Next attribute" }}
              </div>
              <TextPill variant="key2">{{ selectKeyString }}</TextPill>
            </div>
            <div
              v-else
              :class="
                clsx(
                  'text-xs flex-none flex flex-row items-center gap-2xs',
                  themeClasses('text-neutral-900', 'text-neutral-200'),
                )
              "
            >
              <div>Select</div>
              <TextPill variant="key2">{{ selectKeyString }}</TextPill>
            </div>
            <div
              :class="
                clsx(
                  'text-xs flex-none flex flex-row items-center gap-2xs',
                  themeClasses('text-neutral-900', 'text-neutral-200'),
                )
              "
            >
              <div>Navigate</div>
              <TextPill variant="key2">Up</TextPill>
              <TextPill variant="key2">Down</TextPill>
            </div>
          </div>
          <div
            v-if="!isSecret"
            :class="
              clsx(
                'flex flex-row items-center border border-transparent',
                'px-xs py-2xs h-[30px]',
                // Don't show cursor pointer or hover effects for connected arrays/maps
                (props.isArray || props.isMap) && isSetByConnection
                  ? 'cursor-default'
                  : [
                      'cursor-pointer',
                      themeClasses(
                        'hover:border-action-500 active:bg-action-200',
                        'hover:border-action-300 active:bg-action-900',
                      ),
                    ],
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
                  'grow font-mono',
                  !field.state.value &&
                    !isArray && [
                      'italic',
                      themeClasses('text-neutral-600', 'text-neutral-400'),
                    ],
                )
              "
            >
              <template v-if="isArray && !isSetByConnection">
                + Add "{{ displayName }}" item
              </template>
              <template v-else-if="isMap && !mapKey && !isSetByConnection">
                You must enter a key
              </template>
              <template v-else-if="isMap && mapKey && !isSetByConnection">
                "{{ mapKey }}"
              </template>
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
              <TextPill variant="key2">{{ selectKeyString }}</TextPill>
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
            class="scrollable max-h-[10rem]"
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
                      'border-t-neutral-400 hover:border-action-500 active:active:bg-action-200',
                      'border-t-neutral-600 hover:border-action-300 active:active:bg-action-900',
                    ),
                  )
                "
                @click.left="() => selectOption(option)"
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
                  <TextPill variant="key2">{{ selectKeyString }}</TextPill>
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
              Or subscribe to an existing prop
            </div>
            <!--
              Attach the virtualizer to this element. It will use the width and height of
              this element, and will own the scrollbar.

              This will allow us to only create HTML elements for visible items, and speeds up
              the rendering and initialization of the list.
            -->
            <div
              ref="filteredConnectionsListRef"
              class="scrollable max-h-[10rem]"
            >
              <!-- Create a relative-positioned container so that children are relative to its (0,0) -->
              <div
                :class="clsx('relative w-full')"
                :style="{
                  height: `${virtualFilteredConnectionsHeight}px`,
                }"
              >
                <!-- position this item exactly where the virtualizer tells it to go -->
                <template
                  v-for="virtualItem in virtualFilteredConnectionItemsList"
                  :key="
                    filteredConnections[virtualItem.index]?.showAllButton
                      ? 'show-all-button'
                      : filteredConnections[virtualItem.index]
                          ?.possibleConnection?.attributeValueId
                  "
                >
                  <AttributeInputPossibleConnection
                    v-if="
                      filteredConnections[virtualItem.index]?.possibleConnection
                    "
                    :connection="
                      filteredConnections[virtualItem.index]?.possibleConnection
                    "
                    :isConnectionSelected="
                      isConnectionSelected(virtualItem.index)
                    "
                    :virtualItemIndex="virtualItem.index"
                    :virtualItemSize="virtualItem.size"
                    :virtualItemStart="virtualItem.start"
                    @selectConnection="(index) => selectConnection(index)"
                  />
                  <div
                    v-else
                    :class="
                      clsx(
                        `absolute top-0 left-0 w-full h-[${virtualItem.size}px]`,
                        'flex flex-row items-center gap-xs justify-center',
                        'px-xs py-2xs w-full text-center border border-transparent',
                        'text-xs cursor-pointer group',
                        isConnectionSelected(virtualItem.index) && [
                          'input-selected-item',
                          themeClasses('bg-action-200', 'bg-action-900'),
                        ],
                        themeClasses(
                          'hover:border-action-500 active:active:bg-action-200',
                          'hover:border-action-300 active:active:bg-action-900',
                        ),
                      )
                    "
                    :style="{
                      transform: `translateY(${virtualItem.start}px)`,
                    }"
                    @click.left="selectConnection(virtualItem.index)"
                  >
                    <TruncateWithTooltip
                      :class="
                        clsx(
                          'italic',
                          themeClasses('text-neutral-600', 'text-neutral-400'),
                        )
                      "
                    >
                      <template v-if="filteredConnections.length - 1 > 1">
                        Showing {{ filteredConnections.length - 1 }} suggested
                        connections
                      </template>
                      <template
                        v-else-if="filteredConnections.length - 1 === 1"
                      >
                        Showing one suggested connection
                      </template>
                      <template v-else>
                        No suggested subscriptions available
                      </template>
                    </TruncateWithTooltip>
                    <TruncateWithTooltip
                      :class="
                        clsx(
                          'font-bold group-hover:underline',
                          themeClasses(
                            'group-hover:text-action-500',
                            'group-hover:text-action-300',
                          ),
                          isConnectionSelected(virtualItem.index) && [
                            'underline',
                            themeClasses('text-action-500', 'text-action-300'),
                          ],
                        )
                      "
                    >
                      Show All Possible Subscriptions
                    </TruncateWithTooltip>
                  </div>
                </template>
              </div>
            </div>
          </template>

          <!-- display potential connection value area -->
          <div
            v-if="
              selectedConnection?.possibleConnection?.value &&
              (kindAsString === 'textarea' || kindAsString === 'codeeditor')
            "
            class="relative"
          >
            <CodeViewer
              :code="`${JSON.stringify(
                selectedConnection.possibleConnection.value,
                null,
                2,
              )}\n`"
              showTitle
              :allowCopy="false"
              :title="selectedConnection.possibleConnection.path"
            />
          </div>
          <label
            v-if="featureFlagsStore.DEFAULT_SUBS"
            tabindex="0"
            data-default-sub-checkbox="label"
            :for="`checkbox-${prop?.id}`"
            :class="
              clsx(
                'border w-full flex flex-row items-center gap-xs px-xs py-2xs cursor-pointer',
                themeClasses('border-neutral-400', 'border-neutral-600'),
              )
            "
            @click="
              () =>
                toggleIsDefaultSource(
                  `default-source-checkbox-${prop?.id}`,
                  path,
                  true,
                )
            "
          >
            <input
              :id="`default-source-checkbox-${prop?.id}`"
              data-default-sub-checkbox="input"
              type="checkbox"
              :checked="isDefaultSource"
              @click.stop="
                () =>
                  toggleIsDefaultSource(
                    `default-source-checkbox-${prop?.id}`,
                    path,
                    false,
                  )
              "
            />
            <div>Make this the default subscription for new components</div>
          </label>
        </div>
      </template>
      <CodeEditorModal
        ref="codeEditorModalRef"
        :title="`Set Value For ${displayName}`"
        :codeEditorId="`${displayName}-${prop?.id}`"
        @submit="setValueFromCodeEditorModal"
      />
    </template>
  </valueForm.Field>
</template>

<script setup lang="ts">
import {
  computed,
  ComputedRef,
  inject,
  nextTick,
  reactive,
  ref,
  watch,
} from "vue";
import { debounce } from "lodash-es";
import clsx from "clsx";
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
  NewButton,
} from "@si/vue-lib/design-system";
import { Fzf } from "fzf";
import { useQuery, useMutation, useQueryClient } from "@tanstack/vue-query";
import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  PropertyEditorPropWidgetKind,
  PropertyEditorPropWidgetKindComboBox,
  PropertyEditorPropWidgetKindSecret,
  PropertyEditorPropWidgetKindSelect,
  ValidationOutput,
} from "@/api/sdf/dal/property_editor";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import {
  AttributeTree,
  BifrostComponent,
  ComponentInList,
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
import { CategorizedPossibleConnections } from "@/workers/types/dbinterface";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  attributeEmitter,
  MouseDetails,
  mouseEmitter,
} from "../logic_composables/emitters";
import { useWatchedForm } from "../logic_composables/watched_form";
import AttributeValueBox from "./AttributeValueBox.vue";
import CodeEditorModal from "../CodeEditorModal.vue";
import { findAttributeValueInTree } from "../util";
import AttributeInputPossibleConnection from "./AttributeInputPossibleConnection.vue";
import AttributeInputRequiredProperty from "./AttributeInputRequiredProperty.vue";
import { assertIsDefined, AttributeInputContext } from "../types";
import { AttributeErrors } from "../AttributePanel.vue";

type UIConnectionRow = {
  showAllButton?: boolean;
  possibleConnection?: UIPotentialConnection;
};

export type UIPotentialConnection = PossibleConnection & {
  pathArray: string[];
};

const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps<{
  path: AttributePath;
  value: string;
  kind?: null | PropertyEditorPropWidgetKind | string;
  prop?: null | Prop;
  validation?: null | ValidationOutput;
  component: BifrostComponent | ComponentInList;
  displayName: string;
  canDelete?: boolean;
  disabled?: boolean;
  externalSources?: null | ExternalSource[];
  isArray?: boolean;
  isMap?: boolean;
  isSecret?: boolean;
  isDefaultSource?: boolean;
  disableInputWindow?: boolean;
  forceReadOnly?: boolean;
  hasSocketConnection?: boolean;
}>();

const attributeInputContext = inject<AttributeInputContext>("ATTRIBUTEINPUT");

const externalSources = computed(() => {
  if (!props.externalSources) return undefined;
  if (attributeInputContext?.blankInput) return [] as ExternalSource[];
  else return props.externalSources;
});

const showAllPossibleConnections = ref(false);

const isSetByConnection = computed(
  () => props.externalSources && props.externalSources.length > 0,
);

const kindAsString = computed(() => `${props.prop?.widgetKind}`.toLowerCase());

const isPendingValue = computed(
  () =>
    props.externalSources &&
    props.externalSources.length > 0 &&
    props.value === "",
);

const validationStatus = computed(
  (): "passing" | "missingRequiredValue" | "failing" => {
    const failing =
      props.validation &&
      props.validation.status !== "Success" &&
      !isPendingValue.value;
    if (!failing) return "passing";
    if (props.validation.message === '"value" is required')
      return "missingRequiredValue";
    return "failing";
  },
);

// does not set the actual key, just the string displayed!
const selectKeyString = "Tab";

const anchorRef = ref<InstanceType<typeof HTMLElement>>();

type AttrData = { value: string };
const wForm = useWatchedForm<AttrData>(
  `component.av.prop.${props.component.id}.${props.path}`,
  attributeInputContext?.blankInput,
);
// this gets used by the watcher to ensure that data has propagated
const rawAttrData = computed<AttrData>(() => {
  return { value: props.value };
});
// this is used by the form & checks for submission
const attrData = computed<AttrData>(() => {
  if (attributeInputContext?.blankInput) return { value: "" };
  return { value: props.value };
});

const errorContext = inject<ComputedRef<AttributeErrors>>("ATTRIBUTE_ERRORS");
assertIsDefined<ComputedRef<AttributeErrors>>(errorContext);

const errorValue = computed(() => {
  const key = `${props.component.id}-${props.path}`;
  return errorContext.value.saveErrors.value[key];
});
const hasError = computed(() => {
  return !!errorValue.value;
});

const valueForm = wForm.newForm({
  data: attrData,
  onSubmit: async ({ value }) => {
    if (!props.prop) return;
    if (connectingComponentId.value && selectedConnectionData.value) {
      // For new subscriptions, send the raw path to the API, not the display value
      const apiValue = selectedConnectionData.value.propPath;
      emit(
        "save",
        props.path,
        apiValue,
        props.prop.kind,
        connectingComponentId.value,
      );
    } else {
      // For manual values or other cases, use the form value
      emit("save", props.path, value.value, props.prop.kind);
    }
  },
  watchFn: () => {
    return [rawAttrData.value, props.externalSources];
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

  if (props.kind === "checkbox") {
    return {
      hasOptions: true,
      options: [
        { label: "true", value: "true" },
        { label: "false", value: "false" },
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

attributeEmitter.on("selectedPath", ({ path }) => {
  if (path !== props.path) {
    closeInput();
  }
});

const focus = () => {
  attributeEmitter.emit("selectedPath", {
    path: props.path,
    name: props.displayName,
  });
  attributeEmitter.emit("selectedDocs", {
    link: props.prop?.docLink ?? "",
    docs: props.prop?.documentation ?? "",
  });
  openInput();
};

const connectingComponentId = ref<string | undefined>();
const selectedConnectionData = ref<
  { componentName: string; propPath: string } | undefined
>();
const queryClient = useQueryClient();
const makeKey = useMakeKey();

const createSubscriptionMutation = useMutation({
  mutationFn: async (variables: {
    path: AttributePath;
    apiValue: string;
    propKind: PropKind;
    connectingComponentId: ComponentId;
  }) => {
    // Emit to the save handler which calls the actual API
    if (props.prop) {
      emit(
        "save",
        variables.path,
        variables.apiValue,
        variables.propKind,
        variables.connectingComponentId,
      );
    }
    return variables;
  },
  onMutate: async (variables) => {
    const queryKey = makeKey(EntityKind.AttributeTree, props.component.id);

    const previousData = queryClient.getQueryData<AttributeTree>(
      queryKey.value,
    );

    queryClient.setQueryData(
      queryKey.value,
      (cachedData: AttributeTree | undefined) => {
        if (!cachedData) return cachedData;

        const found = findAttributeValueInTree(cachedData, variables.path);
        if (!found || !selectedConnectionData.value) return cachedData;

        const updatedData = { ...cachedData };
        const updatedFound = findAttributeValueInTree(
          updatedData,
          variables.path,
        );
        if (updatedFound) {
          updatedFound.attributeValue.externalSources = [
            {
              componentId: updatedData.id,
              componentName: selectedConnectionData.value.componentName,
              path: selectedConnectionData.value.propPath,
              isSecret: false,
            },
          ];
          updatedFound.attributeValue.value = `subscribing to ${selectedConnectionData.value.propPath}`;
        }

        return updatedData;
      },
    );

    return { previousData };
  },
  onError: (_error, _variables, context) => {
    if (context?.previousData) {
      const queryKey = makeKey(EntityKind.AttributeTree, props.component.id);
      queryClient.setQueryData(queryKey.value, context.previousData);
    }
  },
});

const selectConnection = (index: number) => {
  if (readOnly.value) return;

  const newConnectionRow = filteredConnections.value[index];
  if (!newConnectionRow) return;
  const newConnection = newConnectionRow.possibleConnection;
  if (!newConnection) {
    // clicked the button to show hidden options
    showAllPossibleConnections.value = true;
    cancelTabBehavior.value = true;
    return;
  }

  const apiValue = newConnection.path;
  connectingComponentId.value = newConnection.componentId;
  selectedConnectionData.value = {
    componentName: newConnection.componentName,
    propPath: newConnection.path,
  };

  if (
    apiValue &&
    connectingComponentId.value &&
    apiValue !== attrData.value.value &&
    props.prop
  ) {
    createSubscriptionMutation.mutate({
      path: props.path,
      apiValue,
      propKind: props.prop.kind,
      connectingComponentId: connectingComponentId.value,
    });
  }
  closeInput();
};
const selectOption = (option: LabelEntry<AttrOption>) => {
  if (readOnly.value) return;

  const newValue = option.value.toString();
  connectingComponentId.value = undefined;
  selectedConnectionData.value = undefined;
  if (newValue !== attrData.value.value) {
    valueForm.setFieldValue("value", newValue);
    valueForm.handleSubmit();
  }
  closeInput();
};
const selectDefault = () => {
  if (readOnly.value) return;

  // Don't allow adding items to arrays/maps that are connected via external sources
  if ((props.isArray || props.isMap) && isSetByConnection.value) {
    return;
  }

  const newValue = valueForm.state.values.value;
  connectingComponentId.value = undefined;
  selectedConnectionData.value = undefined;

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
};

const blur = (event: FocusEvent) => {
  if (
    event.relatedTarget instanceof HTMLElement &&
    (event.relatedTarget as HTMLElement).dataset.defaultSubCheckbox
  ) {
    inputRef.value?.focus({ preventScroll: true });
  } else {
    inputRef.value?.focus();
  }
};

const bifrostingTrash = ref(false);
const remove = () => {
  emit("delete", props.path);
  bifrostingTrash.value = true;
};
const removeSubscription = () => {
  emit("removeSubscription", props.path);
};

// TODO add spinner for deletion
const emit = defineEmits<{
  (
    e: "save",
    path: AttributePath,
    value: string,
    propKind: PropKind,
    connectingComponentId?: ComponentId,
  ): void;
  (e: "delete", path: AttributePath): void;
  (e: "removeSubscription", path: AttributePath): void;
  (
    e: "setDefaultSubscriptionSource",
    path: AttributePath,
    setTo: boolean,
  ): void;
  (e: "add", key?: string): void;
  (e: "selected"): void;
  (e: "close"): void;
  (e: "handleTab", event: KeyboardEvent, currentFocus?: HTMLElement): void;
}>();

// INPUT WINDOW LOGIC

const mapKey = ref("");
const mapKeyError = ref(false);
const defaultSelectedIndex = () => -1;
/**
 * The index of the selected option or connection.
 *
 * This is an index into an imagined concatenated list containing both filteredOptions and filteredConnections.
 *
 * - If this is -1, nothing is selected, we start here!
 * - If this is 0, the manual value is selected.
 * - If this is > 0 and <= filteredOptions.length, it references filteredOptions[selectedIndex - 1].
 * - If this is > filteredOptions.length, it references filteredConnetions[selectedIndex - 1 - filteredOptions.length].
 */
const selectedIndex = ref(defaultSelectedIndex());
const inputRef = ref<InstanceType<typeof HTMLInputElement>>();
const inputWindowRef = ref<InstanceType<typeof HTMLDivElement>>();
const inputOpen = ref(false);
const labelRect = ref<undefined | DOMRect>(undefined);

const inputTouched = ref(false);

const resetEverything = () => {
  // Don't reset form state for readonly fields
  // as this can trigger a value form update
  if (readOnly.value) return;

  resetFilteredOptions();
  if (!valueForm.state.canSubmit || valueForm.state.isDirty)
    wForm.reset(valueForm);
  mapKey.value = "";
  mapKeyError.value = false;
  selectedIndex.value = defaultSelectedIndex();
  connectingComponentId.value = undefined;
  selectedConnectionData.value = undefined;
  inputTouched.value = false;
  showAllPossibleConnections.value = false;
  cancelTabBehavior.value = false;
};

const openInput = () => {
  if (readOnly.value || inputOpen.value) return;

  if (props.disableInputWindow) {
    emit("selected");
    return;
  }

  resetEverything();
  labelRect.value = anchorRef.value?.getClientRects()[0];
  if (!labelRect.value) return;
  inputOpen.value = true;
  nextTick(() => {
    inputRef.value?.focus();
    addListeners();
    // fixWindowPosition();
  });
};
const closeInput = () => {
  if (inputOpen.value) {
    inputOpen.value = false;
    emit("close");
    removeListeners();
  }
};
const closeAndReset = () => {
  closeInput();
  resetEverything();
};

const addListeners = () => {
  mouseEmitter.on("mousedown", onMouseDown);
};
const removeListeners = () => {
  mouseEmitter.off("mousedown", onMouseDown);
};

const selectAtCurrentIndex = () => {
  if (readOnly.value) {
    return;
  } else if (selectedIndex.value === -1) {
    closeAndReset();
  } else if (selectedIndex.value === 0) {
    selectDefault();
  } else if (optionIsSelected.value) {
    const option = filteredOptions[selectedIndex.value - 1];
    if (option) {
      selectOption(option);
    }
  } else {
    selectConnection(selectedIndex.value - filteredOptions.length - 1);
  }
};
const onInputChange = (e: Event) => {
  inputTouched.value = true;

  const v = (e.target as HTMLInputElement).value;
  if (props.isMap) {
    mapKey.value = v;
    selectedIndex.value = 0;
  } else {
    valueForm.setFieldValue("value", v);
    filterStr.value = v;
  }

  // fixing various things
  if (props.isArray && v.length === 0 && selectedIndex.value === 0) {
    inputTouched.value = false;
    selectedIndex.value = -1;
  } else if (props.isMap) {
    if (mapKey.value.length === 0) {
      inputTouched.value = false;
      selectedIndex.value = -1;
    } else {
      mapKeyError.value = false;
    }
  } else if (selectedIndex.value === -1) {
    // If the user starts editing the field, move the selector to a value
    if (props.isSecret) {
      selectedIndex.value = 1; // the first connection for secrets
    } else {
      selectedIndex.value = 0; // the manual value for everything else
    }
  }
};
const onMouseDown = (e: MouseDetails["mousedown"]) => {
  const target = e.target;
  if (!(target instanceof Element)) {
    return;
  }
  if (!inputWindowRef.value?.contains(target) && inputOpen.value) {
    // Save the value if it has changed when clicking outside
    if (!readOnly.value && selectedIndex.value === 0) {
      const newValue = valueForm.state.values.value;
      // The newValue has to be different AND this input
      // can't be for an array or map!
      if (
        newValue !== attrData.value.value &&
        !["array", "map"].includes(kindAsString.value)
      ) {
        connectingComponentId.value = undefined;
        selectedConnectionData.value = undefined;
        valueForm.handleSubmit();
        closeInput();
        return;
      }
    }
    closeAndReset();
  }
};
const onUp = (e: KeyboardEvent) => {
  e.preventDefault();

  if (props.isMap) {
    mapArrow();
    return;
  }
  preventAutoScroll.value = false;

  selectedIndex.value--;
  if (selectedIndex.value === 0 && props.isSecret) {
    // A secret field cannot be set via a manual value
    selectedIndex.value = -1;
  }

  if (selectedIndex.value < defaultSelectedIndex()) {
    selectedIndex.value =
      filteredConnections.value.length + filteredOptions.length;
  }
};
const onDown = (e: KeyboardEvent) => {
  e.preventDefault();

  if (props.isMap) {
    mapArrow();
    return;
  }
  preventAutoScroll.value = false;

  selectedIndex.value++;
  if (selectedIndex.value === 0 && props.isSecret) {
    // A secret field cannot be set via a manual value
    selectedIndex.value = 1;
  }

  if (
    selectedIndex.value >
    filteredConnections.value.length + filteredOptions.length
  ) {
    selectedIndex.value = defaultSelectedIndex();
  }
};
const onEnter = () => {
  if (selectedIndex.value === -1) selectedIndex.value = 0;
  selectAtCurrentIndex();
};
const cancelTabBehavior = ref(false);
const inputFocusDivRef = ref<HTMLDivElement>();
const onTab = (e: KeyboardEvent) => {
  if (!readOnly.value) selectAtCurrentIndex();

  if (mapKeyError.value) return;

  if (cancelTabBehavior.value) {
    // This boolean is set to true for one time tab behavior cancellation
    cancelTabBehavior.value = false;
    return;
  }

  // This allows the user to Tab or Shift+Tab to go through the attribute fields
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  const currentFocus = inputFocusDivRef.value;
  if (!currentFocus) return;
  const index = focusable.indexOf(currentFocus);
  if (e.shiftKey) {
    e.preventDefault();
    if (readOnly.value) e.stopPropagation();
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
    if (readOnly.value) {
      e.preventDefault();
      e.stopPropagation();
      nextTick(() => {
        focusable[index + 1]?.focus();
      });
    }
  }
};
const onDelete = (e: KeyboardEvent) => {
  if (!props.canDelete) return;

  if (e.metaKey || e.ctrlKey) {
    e.preventDefault();
    remove();
  }
};

const mapArrow = () => {
  // both arrows do the same thing for Map
  if (selectedIndex.value === -1) {
    selectedIndex.value = 0;
  } else {
    selectedIndex.value = -1;
  }
};

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
        !["string", "checkbox", "object", "map", "integer"].includes(
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
  const output: UIConnectionRow[] = [];

  if (potentialConnQuery.data.value) {
    const addToArray = (
      matches: PossibleConnection[],
      array: UIConnectionRow[],
    ) => {
      // Node(victor): We know that secret props on secret defining schemas live on /secrets/kind name
      // This MAY match other secret props on random schemas, but we check the types match. Ideally the MVs at some
      // point should tells us what props are the secret props on the secret defining schemas. But this solves
      // our current UI hurdle - only suggesting valid secrets as connection sources for secret props
      if (props.isSecret) {
        matches = matches.filter(
          (m) =>
            secretKind.value &&
            m.path === `/secrets/${secretKind.value}` &&
            m.isOriginSecret,
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
        array.push({
          possibleConnection: {
            ...match,
            pathArray,
          },
        });
      });
    };

    addToArray(categorizedPossibleConn.value.suggestedMatches, output);
    if (showAllPossibleConnections.value || props.isSecret) {
      addToArray(categorizedPossibleConn.value.typeAndNameMatches, output);
      addToArray(categorizedPossibleConn.value.typeMatches, output);
    } else {
      output.push({
        showAllButton: true,
      });
    }
  }

  // For arrays and maps, when showing all possible connections
  // we need to filter out any possible connections that don't match kind
  if (
    !output[0]?.showAllButton &&
    (kindAsString.value === "array" || kindAsString.value === "map")
  ) {
    return output.filter(
      (item) => item.possibleConnection?.kind === kindAsString.value,
    );
  }

  return output;
});

const preventAutoScroll = ref(false);

watch(
  () => selectedIndex.value,
  () => {
    nextTick(() => {
      if (preventAutoScroll.value) return;

      if (optionIsSelected.value) {
        const el = document.getElementsByClassName("input-selected-item")[0];
        if (el) {
          el.scrollIntoView({ block: "nearest" });
        }
      } else if (connectionIsSelected.value) {
        virtualFilteredConnections.value.scrollToIndex(
          selectedIndex.value - filteredOptions.length - 1,
        );
      }
    });
  },
);

// Virtualized list for potential connections
const filteredConnectionsListRef = ref<HTMLDivElement>();
const virtualFilteredConnections = useVirtualizer(
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
const virtualFilteredConnectionItemsList = computed(() =>
  virtualFilteredConnections.value.getVirtualItems(),
);
const virtualFilteredConnectionsHeight = computed(() =>
  virtualFilteredConnections.value.getTotalSize(),
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
  () =>
    !!(props.prop?.createOnly && props.component.hasResource) ||
    props.component.toDelete ||
    props.forceReadOnly,
);

const inputHtmlTag = computed(() => {
  if (
    kindAsString.value === "textarea" ||
    kindAsString.value === "codeeditor"
  ) {
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

const toggleIsDefaultSource = (
  checkboxId: string,
  path: AttributePath,
  invertBox: boolean,
) => {
  const checkboxElement = document.getElementById(checkboxId);
  if (!checkboxElement) {
    return;
  }
  const checked = (checkboxElement as HTMLInputElement).checked;
  // If the checkbox input element is clicked the box will be in the real value,
  // if the label is clicked, it will not yet be set to the users intention
  const newValue = invertBox ? !checked : checked;

  emit("setDefaultSubscriptionSource", path, newValue);
};

const optionIsSelected = computed(
  () => selectedIndex.value < filteredOptions.length + 1,
);
const connectionIsSelected = computed(
  () => !optionIsSelected.value && selectedIndex.value > 0,
);

const discardString = computed(() => {
  if (props.isMap) return "Discard key";

  return "Discard edits";
});

const deleteButtonRef = ref<InstanceType<typeof NewButton>>();

const onDeleteButtonTab = (e: KeyboardEvent) => {
  emit("handleTab", e, deleteButtonRef.value?.mainElRef);
};

defineExpose({
  openInput,
  closeInput,
});
</script>
