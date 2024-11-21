<template>
  <div
    v-if="!isHidden"
    :class="
      clsx(
        'attributes-panel-item',
        'relative text-sm',
        isFocus && '--focus',
        isHover && '--hover',
        isSectionHover && '--section-hover',
        canHaveChildren ? '--section' : '--input',
        canHaveChildren && (isOpen ? '--open' : '--collapsed'),
      )
    "
  >
    <!-- SECTION -->
    <div
      v-if="canHaveChildren"
      @mouseleave="onSectionHoverEnd"
      @mouseover.stop="onSectionHoverStart"
    >
      <!-- HEADER -->
      <div
        :style="{
          top: topPx,
          zIndex: headerZIndex,
        }"
        :class="
          clsx('attributes-panel-item__section-header-wrap', 'sticky h-6')
        "
      >
        <div
          :class="
            clsx(
              'attributes-panel-item__section-toggle',
              'absolute w-6 h-6 transition-all duration-200',
              themeClasses(
                'bg-neutral-100 text-neutral-700',
                'bg-neutral-900 text-shade-0',
              ),
              headerHasContent && 'cursor-pointer',
            )
          "
          @click="toggleOpen()"
        >
          <Icon
            :name="
              headerHasContent
                ? isOpen
                  ? 'chevron--down'
                  : 'chevron--right'
                : 'none'
            "
            size="inherit"
            class="opacity-80 hover:opacity-100 hover:scale-110"
          />
        </div>

        <div
          :style="{ marginLeft: indentPx }"
          :class="
            clsx(
              'attributes-panel-item__section-header',
              'h-[inherit] flex flex-row gap-2xs items-center select-none pr-2xs border-b',
              themeClasses(
                isSectionHover ? 'bg-neutral-900' : 'bg-neutral-500',
                isSectionHover
                  ? 'bg-neutral-300 text-shade-100'
                  : 'bg-neutral-600 text-shade-0',
              ),
              themeClasses('text-shade-0 border-shade-0', 'border-neutral-800'),
            )
          "
          @click="toggleOpen(true)"
        >
          <Icon
            v-if="isChildOfMap || isChildOfArray"
            class="attributes-panel-item__nested-arrow-icon w-[14px] h-[14px]"
            name="nested-arrow-right"
            size="none"
          />
          <Icon
            :name="icon"
            class="attributes-panel-item__type-icon"
            size="none"
          />
          <div
            :class="
              clsx(
                'attributes-panel-item__section-header-label',
                'flex flex-row flex-1 items-center whitespace-nowrap gap-2xs min-w-0',
              )
            "
          >
            <div
              ref="headerMainLabelRef"
              v-tooltip="headerMainLabelTooltip"
              :class="
                clsx(
                  'attributes-panel-item__section-header-label-main',
                  'leading-loose flex grow truncate basis-0',
                )
              "
            >
              <template v-if="isChildOfArray">
                {{ propName }}[{{ treeDef.arrayIndex }}]
              </template>
              <template v-else-if="isChildOfMap">
                {{ treeDef.mapKey }}
              </template>
              <template v-else>
                {{ fullPropDef.name }}
              </template>
              <template
                v-if="propIsEditable && (isChildOfArray || isChildOfMap)"
              >
                <button
                  v-tooltip="'Delete'"
                  class="hover:scale-125 hover:text-destructive-500 items-center pl-2xs z-30 flex-none"
                  @click="removeChildHandler"
                >
                  <Icon name="trash" size="xs" />
                </button>
              </template>
            </div>

            <div
              v-if="isMap || isArray"
              :class="
                clsx(
                  'attributes-panel-item__section-header-child-count',
                  'italic mr-2xs text-xs opacity-50 flex-none',
                )
              "
            >
              <template v-if="treeDef.children.length === 0">(empty)</template>
              <template v-else-if="treeDef.children.length === 1"
                >(1 item)</template
              >
              <template v-else>({{ treeDef.children.length }} items)</template>
            </div>
          </div>
          <SourceIconWithTooltip
            v-if="
              !(widgetKind === 'secret') && !props.isRootProp && attributesPanel
            "
            :icon="sourceIcon"
            :overridden="sourceOverridden"
            :tooltipText="sourceTooltip"
            header
          />
          <!-- DROPDOWN MENU FOR SELECT SOURCE -->
          <template
            v-if="attributesPanel && validAttributeValueSources.length > 1"
          >
            <div
              class="attributes-panel-item__section-header-source-select"
              @click="sourceSelectMenuRef?.open($event)"
            >
              <div>set:</div>
              <div
                class="flex flex-row items-center border pl-2xs pr-3xs h-4 text-xs"
              >
                <div class="flex-none whitespace-nowrap">{{ propSource }}</div>
                <Icon name="chevron--down" size="sm" />
              </div>
            </div>

            <DropdownMenu ref="sourceSelectMenuRef">
              <template
                v-for="source in validAttributeValueSources"
                :key="source"
              >
                <DropdownMenuItem
                  :checked="propSource === source"
                  :label="source"
                  checkable
                  @click="setSource(source)"
                />
              </template>
            </DropdownMenu>
          </template>
        </div>
      </div>

      <!-- LEFT BORDER LINE -->
      <div
        v-show="isOpen && headerHasContent"
        :style="{ marginLeft: indentPx, zIndex: headerZIndex }"
        :class="
          clsx(
            'attributes-panel-item__left-border',
            'absolute w-[1px] top-0 bottom-0 pointer-events-none',
            themeClasses(
              isSectionHover ? 'bg-neutral-900' : 'bg-neutral-500',
              isSectionHover ? 'bg-neutral-300' : 'bg-neutral-600',
            ),
          )
        "
      />

      <!-- CHILDREN -->
      <div
        v-show="isOpen && headerHasContent"
        class="attributes-panel-item__children"
      >
        <TreeFormItem
          v-for="childProp in treeDef.children"
          :key="`${propName}/${childProp.propDef?.name}`"
          :treeDef="childProp"
          :level="level + 1"
          :context="context"
          :attributesPanel="attributesPanel"
        />

        <div
          v-if="numberOfHiddenChildren > 0"
          class="attributes-panel-item relative"
        >
          <!-- TODO(wendy) - If we want to add the option to show the hidden props, add the click handler here! -->
          <div
            :style="{ paddingLeft: indentPxPlusOne }"
            class="text-center pt-2xs italic text-2xs text-neutral-400"
          >
            +{{ numberOfHiddenChildren }} hidden empty prop{{
              numberOfHiddenChildren > 1 ? "s" : ""
            }}
          </div>
        </div>

        <template v-if="(isArray || isMap) && propManual">
          <div
            :style="{ marginLeft: indentPx }"
            class="attributes-panel-item__add-child-row"
          >
            <Icon
              class="attributes-panel-item__nested-arrow-icon w-[14px] h-[14px]"
              name="nested-arrow-right"
              size="none"
            />

            <input
              v-if="isMap"
              v-model="newMapChildKey"
              :class="
                clsx(
                  'border w-[150px] h-[28px] min-w-[80px] shrink text-sm p-2xs',
                  themeClasses(
                    'bg-neutral-100 focus:bg-shade-0',
                    'bg-neutral-900 focus:bg-shade-100',
                  ),
                  isMapKeyError
                    ? 'border-destructive-500 focus:border-destructive-500'
                    : themeClasses(
                        'border-neutral-400 focus:border-action-500',
                        'border-neutral-600 focus:border-action-300',
                      ),
                )
              "
              placeholder="key"
              type="text"
              @blur="clearKeyError"
              @keyup.enter="addChildHandler"
            />

            <button
              class="attributes-panel-item__new-child-button items-center"
              @click="addChildHandler"
            >
              <Icon name="plus" size="none" />
              {{ isArray ? "Add array item" : "Add map item" }}
            </button>
          </div>

          <div
            v-if="isMap && isMapKeyError"
            :style="{ marginLeft: indentPx }"
            class="pl-8 pb-2xs italic text-destructive-500"
          >
            You must enter a valid key.
          </div>
        </template>
      </div>
    </div>

    <!-- INDIVIDUAL PROP INSIDE A SECTION -->
    <div
      v-else
      :style="{ paddingLeft: indentPx }"
      :class="
        clsx(
          'attributes-panel-item__item-inner',
          'relative flex flex-row items-center w-full pr-xs',
        )
      "
    >
      <div class="attributes-panel-item__item-label">
        <Icon
          v-if="validation && validation.status !== 'Success'"
          :name="showValidationDetails ? 'chevron--down' : 'chevron--right'"
          class="cursor-pointer"
          size="sm"
          tone="error"
          @click="showValidationDetails = !showValidationDetails"
        />

        <Icon
          v-if="isChildOfMap || isChildOfArray"
          class="attributes-panel-item__nested-arrow-icon w-[14px] h-[14px]"
          name="nested-arrow-right"
          size="none"
        />
        <div
          :title="`${propLabelParts[0]}${propLabelParts[1]}`"
          class="attributes-panel-item__item-label-text"
        >
          <template v-if="isChildOfMap">{{ propLabelParts[1] }}</template>
          <template v-else-if="isChildOfArray">
            [{{ props.treeDef.arrayIndex }}]
          </template>
          <template v-else>{{ propLabel }}</template>
        </div>
        <div
          class="flex flex-row gap-2xs mr-2xs flex-none ml-auto items-center [&>*]:cursor-pointer"
        >
          <button
            v-if="isChildOfMap || isChildOfArray"
            v-tooltip="'Delete'"
            class="hover:text-destructive-500 hover:scale-125 z-30 flex-none"
            @click="removeChildHandler"
          >
            <Icon name="trash" size="xs" />
          </button>

          <SourceIconWithTooltip
            v-if="!(widgetKind === 'secret') && attributesPanel"
            :icon="sourceIcon"
            :overridden="sourceOverridden"
            :tooltipText="sourceTooltip"
          />

          <a
            v-if="docLink"
            v-tooltip="'View Documentation'"
            :href="docLink"
            class="attributes-panel-item__docs-icon hover:scale-125"
            target="_blank"
            title="show docs"
          >
            <Icon
              :class="
                clsx(
                  'attributes-panel-item__help-icon',
                  'text-neutral-400 p-[3px] cursor-pointer hover:text-shade-0',
                )
              "
              name="docs"
            />
          </a>
        </div>
      </div>

      <div
        :class="{
          'force-border-destructive-500':
            validation && validation.status !== 'Success',
          'my-1': validation && validation.status !== 'Success',
        }"
        class="attributes-panel-item__input-wrap group/input"
        @mouseleave="onHoverEnd"
        @mouseover="onHoverStart"
      >
        <Icon
          v-if="
            noValue && !iconShouldBeHidden && !isFocus && !propPopulatedBySocket
          "
          :name="icon"
          class="attributes-panel-item__type-icon"
          size="sm"
        />
        <Icon
          v-if="unsetButtonShow"
          :class="
            clsx(
              'absolute top-0 w-[28px] h-[28px] p-[3px] opacity-50 hover:opacity-100 cursor-pointer z-[2]',
              !canHaveChildren && (isHover || isFocus) ? 'block' : 'hidden',
              widgetKind === 'select' || widgetKind === 'comboBox'
                ? 'right-5'
                : 'right-0',
              `widget-${widgetKind}`,
            )
          "
          name="x-circle"
          @click="unsetHandler"
        />
        <Icon
          v-if="validation"
          :class="
            clsx(
              'absolute top-3xs',
              unsetButtonShow ? 'group-hover/input:right-6 ' : '',
              ['comboBox', 'select'].includes(widgetKind)
                ? 'right-6'
                : 'right-0',
            )
          "
          :name="validation?.status === 'Success' ? 'check' : 'x'"
          :tone="validation?.status === 'Success' ? 'success' : 'error'"
        />
        <template v-if="propKind === 'integer'">
          <input
            v-model="newValueNumber"
            :disabled="!propIsEditable"
            spellcheck="false"
            type="number"
            @blur="onBlur"
            @focus="onFocus"
            @keyup.enter="updateValue"
          />
        </template>
        <template v-else-if="widgetKind === 'text'">
          <input
            v-model="newValueString"
            :class="`${propLabelParts[0]}${propLabelParts[1]}`"
            :disabled="!propIsEditable"
            spellcheck="false"
            type="text"
            @blur="onBlur"
            @focus="onFocus"
            @keyup.enter="updateValue"
          />
        </template>
        <template v-else-if="widgetKind === 'password'">
          <!-- todo add show/hide controls -->
          <input
            v-model="newValueString"
            :disabled="!propIsEditable"
            type="password"
            @blur="onBlur"
            @focus="onFocus"
            @keyup.enter="updateValue"
          />
        </template>
        <template
          v-else-if="widgetKind === 'textArea' || widgetKind === 'codeEditor'"
        >
          <textarea
            v-model="newValueString"
            :class="`$propLabelParts`"
            :disabled="!propIsEditable"
            spellcheck="false"
            @blur="onBlur"
            @focus="onFocus"
            @keydown.enter="(e) => e.metaKey && updateValue()"
          />
          <Icon
            v-if="propControlledByParent"
            class="attributes-panel-item__popout-view-button"
            name="external-link"
            title="View in popup"
            @click="viewModalRef?.open()"
          />
          <Icon
            v-else
            class="attributes-panel-item__popout-edit-button"
            name="external-link"
            title="Edit in popup"
            @click="editModalRef?.open()"
          />

          <!-- <button class="attributes-panel-item__popout-edit-button2">
            <Icon name="external-link" size="none" />
            Expand
          </button> -->
        </template>
        <template v-else-if="widgetKind === 'checkbox'">
          <input
            :checked="newValueBoolean"
            :class="`attributes-panel-item__hidden-input ${propLabelParts[0]}${propLabelParts[1]}`"
            :disabled="!propIsEditable"
            type="checkbox"
            @blur="onBlur"
            @change="updateValue"
            @focus="onFocus"
            @input="
              (e) => (newValueBoolean = (e.target as HTMLInputElement)?.checked)
            "
          />
          <div
            :class="
              clsx(
                'attributes-panel-item__input-value-checkbox',
                'flex flex-row px-xs py-[5px] items-center',
              )
            "
          >
            <Icon
              :name="newValueBoolean === true ? 'check-square' : 'empty-square'"
              class="inline-block w-[22px] h-[22px] ml-[-4px] mr-2xs my-[-4px] p-0"
            />
            {{ newValueBoolean ? "TRUE" : "FALSE" }}
          </div>
        </template>
        <template
          v-else-if="widgetKind === 'comboBox' || widgetKind === 'select'"
        >
          <select
            v-model="newValueString"
            :class="`attributes-panel-item__hidden-input ${propLabelParts[0]}${propLabelParts[1]}`"
            :disabled="!propIsEditable"
            @change="updateValue"
            @focus="onFocus"
          >
            <option v-for="o in widgetOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </option>
          </select>
          <div class="attributes-panel-item__input-value-select">
            {{ currentValue }}
          </div>
          <Icon
            class="absolute right-1 top-1 text-neutral-400 dark:text-neutral-600"
            name="input-type-select"
            size="sm"
          />
        </template>
        <template v-else-if="widgetKind === 'secret'">
          <div class="p-2xs" @click="secretModalRef?.open()">
            <div
              v-if="secret"
              :class="
                clsx(
                  'line-clamp-6 px-[10px] py-3xs rounded leading-[18px] text-xs cursor-pointer',
                  secret.isUsable
                    ? themeClasses('bg-action-200', 'bg-action-700')
                    : themeClasses('bg-destructive-400', 'bg-destructive-600'),
                )
              "
            >
              <Icon name="key" size="2xs" class="inline-block align-middle" />
              {{ secret.definition }} / {{ secret.name }}
            </div>
            <div
              v-else
              class="opacity-60 italic pl-6 cursor-pointer hover:opacity-100"
            >
              select/add secret
            </div>
          </div>

          <SecretsModal
            v-if="secretDefinitionIdForProp"
            ref="secretModalRef"
            :definitionId="secretDefinitionIdForProp"
            @select="secretSelectedHandler"
          />
        </template>
        <template v-else>
          <div class="py-[4px] px-[8px] text-sm">{{ widgetKind }}</div>
        </template>
        <div
          v-if="!propIsEditable && attributesPanel"
          v-tooltip="sourceTooltip"
          :class="
            clsx(
              'attributes-panel-item__blocked-overlay',
              'absolute top-0 w-full h-full z-50 text-center flex flex-row items-center justify-center cursor-pointer opacity-50',
              themeClasses('bg-caution-lines-light', 'bg-caution-lines-dark'),
            )
          "
          @click="openConfirmEditModal"
        />
      </div>

      <DetailsPanelMenuIcon v-if="!attributesPanel" class="ml-xs" />
    </div>

    <!-- VALIDATION DETAILS -->
    <div
      v-if="showValidationDetails && validation"
      :class="
        clsx(
          'attributes-panel-item__validation-details flex flex-col p-2xs border mx-xs text-xs translate-y-[-5px] font-mono',
          'text-destructive-500 border-destructive-500',
          themeClasses('bg-destructive-100', 'bg-neutral-900'),
        )
      "
      :style="{ marginLeft: indentPx }"
    >
      {{ validation.message }}

      <!-- no more logs <span
        v-for="(output, index) in validation.logs"
        :key="index"
        class="text-sm break-all text-warning-500"
      >
        <p v-if="output.stream !== 'output'">{{ output.message }}</p>
      </span> -->
    </div>

    <!-- MODAL FOR EDITING A PROP -->
    <Modal
      v-if="widgetKind === 'textArea' || widgetKind === 'codeEditor'"
      ref="editModalRef"
      :title="`Edit value - ${propLabel}`"
      class="attributes-panel-item__edit-value-modal"
      size="4xl"
      @close="updateValue"
    >
      <div
        :class="
          clsx(
            'relative h-[40vh]',
            '[&_.ͼ1.cm-editor.cm-focused]:outline-none [&_.ͼ1.cm-editor]:border',
            themeClasses(
              '[&_.ͼ1.cm-editor]:border-neutral-400 [&_.ͼ1.cm-editor.cm-focused]:border-action-500',
              '[&_.ͼ1.cm-editor]:border-neutral-600 [&_.ͼ1.cm-editor.cm-focused]:border-action-300',
            ),
            themeClasses('bg-shade-0', 'bg-shade-100'),
          )
        "
      >
        <template v-if="widgetKind === 'textArea'">
          <textarea
            v-model="newValueString"
            :class="
              clsx(
                'border bg-transparent w-full h-full overflow-auto absolute text-sm leading-5 font-mono resize-none block',
                themeClasses(
                  'border-neutral-400 focus:border-action-500',
                  'border-neutral-600 focus:border-action-300',
                ),
              )
            "
            spellcheck="false"
          />
        </template>
        <template v-else-if="widgetKind === 'codeEditor'">
          <CodeEditor
            :id="`${changeSetsStore.selectedChangeSetId}/${treeDef.valueId}`"
            v-model="newValueString"
            :recordId="treeDef.valueId"
          />
        </template>
      </div>
      <!-- <VButton @click="editModalRef?.close">Save</VButton> -->
    </Modal>

    <!-- MODAL FOR VIEWING A PROP WHICH CANNOT BE EDITED -->
    <Modal
      v-if="widgetKind === 'textArea' || widgetKind === 'codeEditor'"
      ref="viewModalRef"
      :title="`View value - ${propLabel}`"
      class="attributes-panel-item__view-value-modal"
      size="4xl"
    >
      <div class="pb-xs text-destructive-500 font-bold">
        This value cannot currently be edited because
        {{
          propControlledByParent
            ? "it is being controlled by a parent function."
            : "it is being driven by a socket."
        }}
      </div>
      <div
        :class="
          clsx(
            'relative border max-h-[70vh]',
            widgetKind === 'textArea' ? 'overflow-auto' : 'overflow-hidden',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <template v-if="widgetKind === 'textArea'">
          <pre class="font-mono block overflow-auto p-xs"
            >{{ newValueString }}
          </pre>
        </template>
        <template v-else-if="widgetKind === 'codeEditor'">
          <CodeViewer
            :code="newValueString"
            disableScroll
            class="max-h-[70vh]"
          />
        </template>
      </div>
    </Modal>

    <!-- MODAL FOR WHEN YOU CLICK A PROP WHICH IS CONTROLLED BY A PARENT OR SOCKET -->
    <Modal ref="confirmEditModalRef" :title="confirmEditModalTitle">
      <div class="pb-sm">
        <template v-if="propControlledByParent">
          You cannot edit prop "{{ propName }}" because it is populated by a
          function from an ancestor prop.
        </template>
        <template v-else-if="isImmutableSecretProp">
          You cannot edit a non-origin secret prop. You can only edit a secret
          prop on the component that it originates from (i.e. the component
          whose asset defines it). For example, an "AWS Credential" secret must
          be set on an "AWS Credential" component.
        </template>
        <template v-else>
          Editing the prop "{{ propName }}" directly will override the value
          that is set by a dynamic function:
          <div class="flex flex-row items-end justify-center">
            <Icon
              :name="clipIcon"
              class="cursor-pointer"
              size="xs"
              @click="copyToClipboard"
            />
            <pre
              class="text-center mt-xs cursor-pointer overflow-y-auto break-all text-wrap max-h-[50vh]"
              @click="copyToClipboard"
              >{{ currentValue }}</pre
            >
          </div>
        </template>
      </div>
      <div class="flex gap-sm">
        <VButton
          :class="
            propControlledByParent || isImmutableSecretProp ? 'flex-grow' : ''
          "
          icon="x"
          tone="shade"
          variant="ghost"
          @click="closeConfirmEditModal"
        >
          Cancel
        </VButton>
        <VButton
          v-if="!propControlledByParent && !isImmutableSecretProp"
          class="flex-grow"
          icon="edit"
          tone="action"
          @click="confirmEdit"
        >
          Confirm
        </VButton>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType, ref, watch } from "vue";
import clsx from "clsx";
import {
  DropdownMenu,
  DropdownMenuItem,
  Icon,
  IconNames,
  Modal,
  themeClasses,
  VButton,
} from "@si/vue-lib/design-system";
import {
  AttributeTreeItem,
  useComponentAttributesStore,
} from "@/store/component_attributes.store";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Secret, useSecretsStore } from "@/store/secrets.store";
import {
  PropertyEditorProp,
  PropertyEditorPropKind,
  PropertyEditorPropWidgetKind,
  PropertyEditorValue,
  ValidationOutput,
} from "@/api/sdf/dal/property_editor";
import TreeFormItem from "./TreeFormItem.vue"; // eslint-disable-line import/no-self-import
import CodeEditor from "../CodeEditor.vue";
import SecretsModal from "../SecretsModal.vue";
import SourceIconWithTooltip from "./SourceIconWithTooltip.vue";
import CodeViewer from "../CodeViewer.vue";
import DetailsPanelMenuIcon from "../DetailsPanelMenuIcon.vue";

export type TreeFormProp = {
  name: string;
  icon: IconNames;
  kind: PropertyEditorPropKind;
  widgetKind: PropertyEditorPropWidgetKind;
  isHidden: boolean;
  isReadonly: boolean;
};

export type TreeFormData = {
  propDef: TreeFormProp;
  children: TreeFormData[];
  value: PropertyEditorValue | undefined;
  valueId: string;
  parentValueId: string;
  validation: ValidationOutput | null;
  propId: string;
  mapKey?: string;
  arrayKey?: string;
  arrayIndex?: number;
};

const props = defineProps({
  treeDef: {
    type: Object as PropType<AttributeTreeItem | TreeFormData>,
    required: true,
  },
  level: { type: Number, default: 0 },
  isRootProp: { type: Boolean, default: false },
  context: { type: Function, required: true },

  // Only set this boolean to true if this TreeFormItem is part of AttributesPanel
  attributesPanel: { type: Boolean },
});

const headerMainLabelRef = ref();
const headerMainLabelTooltip = computed(() => {
  if (!headerMainLabelRef.value) return;

  if (
    headerMainLabelRef.value.clientWidth < headerMainLabelRef.value.scrollWidth
  ) {
    return {
      content: headerMainLabelRef.value.textContent,
    };
  } else return {};
});

const isOpen = ref(true); // ref(props.attributeDef.children.length > 0);
const showValidationDetails = ref(false);

const shouldBeHidden = (item: AttributeTreeItem | TreeFormData) => {
  if (!item.value?.isControlledByAncestor) return false;

  const canHaveChildren = ["object", "map", "array"].includes(
    item.propDef.kind,
  );

  if (canHaveChildren && item.children.length === 0) {
    return true;
  }

  const children = [];
  children.push(item);

  while (children.length > 0) {
    const child = children.pop();
    if (!child) break;
    if (["object", "map", "array"].includes(child.propDef.kind)) {
      _.extend(children, child.children);
    } else if (child.value) {
      return false;
    }
  }
  return true;
};

const isHidden = computed(() => shouldBeHidden(props.treeDef));

const unsetButtonShow = computed(
  () =>
    sourceOverridden.value &&
    currentValue.value !== null &&
    !propPopulatedBySocket.value &&
    !propControlledByParent.value,
);

const numberOfHiddenChildren = computed(() => {
  let count = 0;
  props.treeDef.children.forEach((child) => {
    if (shouldBeHidden(child)) count++;
  });
  return count;
});

const headerHasContent = computed(() => {
  return (
    props.treeDef.children.length ||
    ((isArray.value || isMap.value) && propManual.value)
  );
});

const rootCtx = props.context();

// not reactive - and we know it's populated - since the parent will rerender if it changes
const componentsStore = useComponentsStore();
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const componentId = componentsStore.selectedComponentId!;

const changeSetsStore = useChangeSetsStore();
const attributesStore = useComponentAttributesStore(componentId);
const secretsStore = useSecretsStore();

const fullPropDef = computed(() => props.treeDef.propDef);
const propKind = computed(() => fullPropDef.value.kind);
const widgetKind = computed(() => fullPropDef.value.widgetKind.kind);
const widgetOptions = computed(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  () => (fullPropDef.value.widgetKind as any).options,
);
const propName = computed(() => fullPropDef.value.name);
const propLabelParts = computed(() => {
  if (isChildOfArray.value)
    return [`${propName.value}[${props.treeDef.arrayIndex}]`];
  if (isChildOfMap.value) return [`${propName.value}.`, props.treeDef.mapKey];
  return ["", propName.value];
});
const propLabel = computed(() => propLabelParts.value.join(""));

const isArray = computed(() => propKind.value === "array");
const isMap = computed(() => propKind.value === "map");
const isMapKeyError = ref(false);
const clearKeyError = () => {
  isMapKeyError.value = false;
};
const isChildOfArray = computed(() => props.treeDef.arrayIndex !== undefined);
const isChildOfMap = computed(() => props.treeDef.mapKey !== undefined);

const canHaveChildren = computed(() => {
  return ["object", "map", "array"].includes(propKind.value);
});

const WIDGET_ICON_LOOKUP: Record<string, IconNames> = {
  codeEditor: "brackets-angle",
  // array: "check",
  checkbox: "check",
  // header: "check",
  // map: "check",
  text: "input-type-string",
  textArea: "input-type-text",
  password: "password",
  integer: "input-type-number",
  comboBox: "input-type-select",
  select: "input-type-select",
  secret: "key",
  color: "check",
};

const icon = computed((): IconNames => {
  if (!props.attributesPanel) {
    return (props.treeDef.propDef as TreeFormProp).icon;
  }

  if (propKind.value === "array") return "brackets-square";
  if (propKind.value === "map") return "brackets-curly";
  if (propKind.value === "object") return "bullet-list";
  if (propKind.value === "integer") return "input-type-number";
  return WIDGET_ICON_LOOKUP[widgetKind.value] || "question-circle";
});

const docLink = computed(() =>
  props.attributesPanel
    ? (props.treeDef as AttributeTreeItem).propDef.docLink
    : undefined,
);

const HEADER_HEIGHT = 24;
const INDENT_SIZE = 8;

const indentPx = computed(
  () => `${HEADER_HEIGHT + INDENT_SIZE * props.level}px`,
);
const topPx = computed(() => `${HEADER_HEIGHT * props.level}px`);
const indentPxPlusOne = computed(
  () => `${HEADER_HEIGHT + INDENT_SIZE * (props.level + 1)}px`,
);

const headerZIndex = computed(() => 300 - props.level);

const newMapChildKey = ref("");

const currentValue = computed(() => props.treeDef.value?.value);

const clipIcon = ref<IconNames>("clipboard-copy");
const copyToClipboard = () => {
  navigator.clipboard.writeText(currentValue.value as string);
  clipIcon.value = "check2" as IconNames;
  setTimeout(() => {
    clipIcon.value = "clipboard-copy" as IconNames;
  }, 2000);
};

const newValueBoolean = ref<boolean>();
const newValueString = ref<string>("");
// The input may set the value to an empty string instead of null or undefined when the input is deleted
const newValueNumber = ref<number | null | "">();

const instantValue = computed(() => {
  switch (widgetKind.value) {
    case "integer":
      if (newValueNumber.value === "") {
        return null;
      } else {
        return newValueNumber.value;
      }
    case "checkbox":
      return newValueBoolean.value;
    default:
      return newValueString.value.trim();
  }
});

const noValue = computed(() => {
  return instantValue.value === null && newValueString.value === "";
});
const iconShouldBeHidden = computed(
  () => icon.value === "input-type-select" || icon.value === "check",
);

const propPopulatedBySocket = computed(
  () => props.treeDef.value?.isFromExternalSource,
);
const propHasSocket = computed(() => props.treeDef.value?.canBeSetBySocket);
const propSetByDynamicFunc = computed(
  () =>
    props.treeDef.value?.isControlledByDynamicFunc &&
    !propHasSocket.value &&
    !propPopulatedBySocket.value,
);
const propManual = computed(
  () =>
    !(
      propPopulatedBySocket.value ||
      propHasSocket.value ||
      propSetByDynamicFunc.value
    ),
);

enum AttributeValueSource {
  Manual = "manually",
  Socket = "via socket",
  NonSocketAttributeFunc = "via attribute func",
}

const validAttributeValueSources = computed(() => {
  const sources = [];

  // TODO(victor): Get if default function is dynamic from the api to show NonSocketAttributeFunc option on the dropdown

  if (
    props.attributesPanel &&
    (props.treeDef.propDef as PropertyEditorProp).defaultCanBeSetBySocket
  ) {
    sources.push(AttributeValueSource.Socket);
  }

  if (propControlledByParent.value === false) {
    sources.push(AttributeValueSource.Manual);
  }
  if (!sources.includes(propSource.value)) {
    sources.push(propSource.value);
  }

  return sources;
});

const propSource = computed<AttributeValueSource>(() => {
  if (propHasSocket.value || propPopulatedBySocket.value)
    return AttributeValueSource.Socket;
  else if (propSetByDynamicFunc.value)
    return AttributeValueSource.NonSocketAttributeFunc;
  else return AttributeValueSource.Manual;
});

const setSource = (source: AttributeValueSource) => {
  if (source === AttributeValueSource.Manual) {
    const value = props.treeDef.value?.value ?? null;

    attributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: props.treeDef.valueId,
        parentAttributeValueId: props.treeDef.parentValueId,
        propId: props.treeDef.propId,
        componentId,
        value,
        isForSecret: false,
      },
    });
  } else {
    attributesStore.RESET_PROPERTY_VALUE({
      attributeValueId: props.treeDef.valueId,
    });
  }
};

const sourceIcon = computed(() => {
  if (propPopulatedBySocket.value) return "circle-full";
  else if (propSetByDynamicFunc.value) return "func";
  else if (propHasSocket.value) return "circle-empty";
  else return "cursor";
});

const sourceOverridden = computed(() => props.treeDef.value?.overridden);

const propIsEditable = computed(() => {
  if (isImmutableSecretProp.value) {
    return false;
  }
  return (
    sourceOverridden.value ||
    editOverride.value ||
    (!propPopulatedBySocket.value && !propSetByDynamicFunc.value)
  );
});

const sourceTooltip = computed(() => {
  if (sourceOverridden.value) {
    if (propPopulatedBySocket.value) {
      return `${propName.value} has been overriden to be set via a populated socket`;
    } else if (propSetByDynamicFunc.value) {
      return `${propName.value} has been overriden to be set by a dynamic function`;
    } else if (propHasSocket.value) {
      return `${propName.value} has been overriden to be set via an empty socket`;
    }
    return `${propName.value} has been set manually`;
  } else {
    if (propPopulatedBySocket.value) {
      return `${propName.value} is set via a populated socket`;
    } else if (propControlledByParent.value) {
      return `${propName.value} is set via a function from an ancestor`;
    } else if (propSetByDynamicFunc.value) {
      return `${propName.value} is set by a dynamic function`;
    } else if (propHasSocket.value) {
      return `${propName.value} is set via an empty socket`;
    }
    return `${propName.value} can be set manually`;
  }
});

const propControlledByParent = computed(
  () => props.treeDef.value?.isControlledByAncestor,
);

function resetNewValueToCurrentValue() {
  newValueBoolean.value = !!currentValue.value;
  newValueString.value = currentValue.value?.toString() || "";
  const valAsNumber = parseFloat(currentValue.value?.toString() || "");
  newValueNumber.value = Number.isNaN(valAsNumber) ? undefined : valAsNumber;
  showValidationDetails.value = false;
}
watch(currentValue, resetNewValueToCurrentValue, { immediate: true });

function toggleOpen(newIsOpen?: boolean) {
  if (canHaveChildren.value) {
    if (_.isBoolean(newIsOpen)) isOpen.value = newIsOpen;
    else isOpen.value = !isOpen.value;
  }
}

const newMapChildKeyIsValid = computed(() => {
  if (propKind.value !== "map") return true;
  if (!newMapChildKey.value.trim().length) return false;
  return true;
});

function removeChildHandler() {
  if (!isChildOfArray.value && !isChildOfMap.value) return;

  if (props.attributesPanel) {
    attributesStore.REMOVE_PROPERTY_VALUE({
      attributeValueId: props.treeDef.valueId,
      propId: props.treeDef.propId,
      componentId,
      key: getKey(),
    });
  } else {
    // TODO(Wendy) - make this functional for TreeForm when needed
  }
}

const validation = computed(() => {
  if (widgetKind.value === "secret" && secret.value?.isUsable === false) {
    return {
      status: "Failure",
      message:
        "Unusable Secret: Created in another workspace. Edit it to be able to use it.",
    };
  }

  return props.treeDef?.validation;
});

function getKey() {
  if (isChildOfMap.value) return props.treeDef?.mapKey;

  return props.treeDef?.arrayKey;
}

function addChildHandler() {
  const isAddingMapChild = propKind.value === "map";
  if (isAddingMapChild && !newMapChildKeyIsValid.value) {
    isMapKeyError.value = true;
    return;
  }

  if (props.attributesPanel) {
    attributesStore.UPDATE_PROPERTY_VALUE({
      insert: {
        parentAttributeValueId: props.treeDef.valueId,
        propId: props.treeDef.propId,
        componentId,
        ...(isAddingMapChild && {
          key: newMapChildKey.value.trim(),
        }),
      },
    });
    newMapChildKey.value = "";
  } else {
    // TODO(Wendy) - make this functional for TreeForm when needed
  }
}
function unsetHandler() {
  newValueBoolean.value = false;
  newValueString.value = "";

  if (props.attributesPanel) {
    attributesStore.RESET_PROPERTY_VALUE({
      attributeValueId: props.treeDef.valueId,
    });
  } else {
    // TODO(Wendy) - make this functional for TreeForm when needed
  }
}

function updateValue() {
  let newVal;
  let skipUpdate = false;
  let isForSecret = false;

  if (widgetKind.value === "checkbox") {
    newVal = newValueBoolean.value;
    // special handling for empty value + false
    if (newVal === false && !currentValue.value) skipUpdate = true;
  } else if (propKind.value === "integer") {
    if (newValueNumber.value === "") {
      newVal = null;
    } else {
      newVal = newValueNumber.value;
    }
  } else {
    // for now, we will always trim, but we need to be smarter about this
    // meaning have options, and more generally have some cleaning / coercion logic...
    newValueString.value = newValueString.value.trim();

    newVal = newValueString.value;
    // special handling for empty value + empty string
    if (newVal === "" && !currentValue.value) skipUpdate = true;
  }

  // don't trigger an update if the value has not changed
  // (and some special cases handled for specific types)
  if (skipUpdate || newVal === currentValue.value) {
    return;
  }

  // If we are explicitly setting a secret, we need to inform SDF so that dependent values update
  // will trigger when the secret's encrypted contents change.
  if (widgetKind.value === "secret") {
    isForSecret = true;
  }

  if (props.attributesPanel) {
    attributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: props.treeDef.valueId,
        parentAttributeValueId: props.treeDef.parentValueId,
        propId: props.treeDef.propId,
        componentId,
        value: newVal,
        isForSecret,
      },
    });
  } else {
    // TODO(Wendy) - make this functional for TreeForm
  }
}

const isHover = ref(false);
const isFocus = ref(false);

function onHoverStart() {
  if (!propControlledByParent.value) {
    isHover.value = true;
  }
}
function onHoverEnd() {
  isHover.value = false;
}
function onFocus() {
  isFocus.value = true;
}
function onBlur() {
  isFocus.value = false;
  updateValue();
}
function onSectionHoverStart() {
  isHover.value = true;
  rootCtx.hoverSectionValueId.value = props.treeDef.valueId;
}
function onSectionHoverEnd() {
  isHover.value = false;
  if (rootCtx.hoverSectionValueId.value === props.treeDef.valueId) {
    rootCtx.hoverSectionValueId.value = undefined;
  }
}
const isSectionHover = computed(
  () => rootCtx.hoverSectionValueId.value === props.treeDef.valueId,
);

const viewModalRef = ref<InstanceType<typeof Modal>>();
const editModalRef = ref<InstanceType<typeof Modal>>();
const secretModalRef = ref<InstanceType<typeof SecretsModal>>();

const secret = computed(
  () => secretsStore.secretsById[newValueString.value?.toString() || ""],
);
const isImmutableSecretProp = computed(
  () =>
    props.attributesPanel &&
    !(fullPropDef.value as PropertyEditorProp).isOriginSecret &&
    widgetKind.value === "secret",
);
const secretDefinitionIdForProp = computed(() => {
  if (props.treeDef.propDef.widgetKind.kind !== "secret") return;
  const widgetOptions = props.treeDef.propDef.widgetKind.options;
  // A widget of kind=secret has a single option that points to its secret definition
  const secretKind = _.find(
    widgetOptions,
    (o) => o.label === "secretKind",
  )?.value;
  if (!secretKind) throw new Error("Missing secretKind on secret widget...?");
  return secretKind;
});
function secretSelectedHandler(newSecret: Secret) {
  newValueString.value = newSecret.id;
  updateValue();
  secretModalRef.value?.close();
}

const confirmEditModalRef = ref<InstanceType<typeof Modal>>();
const confirmEditModalTitle = computed(() => {
  if (propControlledByParent.value) {
    if (propName.value) {
      return `You Cannot Edit Prop &quote;${propName.value}&quote;`;
    }
    return "You Cannot Edit Prop";
  }

  if (isImmutableSecretProp.value) {
    return "You Cannot Edit Non-Origin Secret Prop";
  }

  return "Are You Sure?";
});

const openConfirmEditModal = () => {
  if (confirmEditModalRef.value) {
    confirmEditModalRef.value.open();
  }
};

const closeConfirmEditModal = () => {
  if (confirmEditModalRef.value) {
    confirmEditModalRef.value.close();
  }
};

const confirmEdit = () => {
  editOverride.value = true;
  closeConfirmEditModal();
};

const editOverride = ref(false);

const sourceSelectMenuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>

<style lang="less">
// sync these with above
@header-height: 24px;
@indent-size: 8px;

.attributes-panel-item__section-toggle {
  .attributes-panel.--show-section-toggles & {
    opacity: 1;
  }
}

.attributes-panel-item__type-icon {
  height: 100%;
  padding: 2px;
  position: relative;
}

.attributes-panel-item__hidden-input {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  padding: 0;
  height: 100%;
  opacity: 0;
  z-index: 1;
  display: block;
  cursor: pointer;
}

.attributes-panel-item__item-label,
.attributes-panel-item__add-child-row {
  display: flex;
  flex-grow: 1;
  gap: @spacing-px[xs];
  position: relative;
  overflow: hidden;
  align-items: center;
}
.attributes-panel-item__item-label-text {
  cursor: default;
  flex-shrink: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 4px 0; // fixes cut off descenders
  i {
    font-style: normal;
    opacity: 0.5;
  }
}

.force-border-destructive-500 {
  border-color: @colors-destructive-500 !important;
}
.attributes-panel-item__input-wrap {
  position: relative;
  border: 1px solid var(--input-border-color);
  min-height: 30px;
  width: 45%;
  flex-shrink: 0;
  background: var(--input-bg-color);
  font-family: monospace;
  font-size: 13px;
  line-height: 18px;

  .attributes-panel-item.--focus & {
    background: var(--input-focus-bg-color);
    z-index: 101;
  }

  input,
  textarea {
    @apply focus:ring-0 focus:ring-offset-0;
    background: transparent;
    font-family: inherit;
    padding: 5px 8px;
    width: 100%;
    border: none;
    font-size: inherit;
    line-height: inherit;
    display: block;
    text-overflow: ellipsis;
    overflow: hidden;

    .attributes-panel-item.--input.--focus &,
    .attributes-panel-item.--input.--hover & {
      padding-right: 28px; // to give space for unset button
    }
  }
  textarea {
    min-height: 80px;
    margin: 0;
  }

  // chrome + linux showing white on white text - this might fix it?
  select {
    option {
      background: white;
      color: black;
    }
  }

  .attributes-panel-item__type-icon {
    position: absolute;
    left: 0px;
    top: 0px;
    width: 28px;
    height: 28px;
    padding: 3px;
    z-index: 2;
    pointer-events: none;
  }
}

.attributes-panel-item__input-value-select {
  padding: 5px 24px 5px 8px;
  display: block;
  text-overflow: ellipsis;
  overflow: hidden;
}

.attributes-panel-item__action-icons {
  gap: 4px;
  padding: 4px;
  margin-left: 4px;
  margin-right: 4px;

  .attributes-panel-item__item-inner & {
    position: absolute;
    display: none;
    right: 30px;
  }
  .attributes-panel-item__item-inner:hover & {
    display: flex;
  }

  .attributes-panel-item__section-header & {
    display: none;
  }
  .attributes-panel-item__section-header:hover & {
    display: flex;
  }
}

.attributes-panel-item__section-header-source-select {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  cursor: pointer;
}

.attributes-panel-item__section-header-source-select > div {
  .attributes-panel-item__section-header:hover & {
    body.dark & {
      border-color: black;
    }
  }
}

// small icon buttons
.attributes-panel-item__action-icons .icon,
.attributes-panel-item__popout-edit-button,
.attributes-panel-item__popout-view-button,
.attributes-panel-item__new-child-button {
  width: 20px;
  height: 20px;
  padding: 2px;
  position: relative;
  border: 1px solid currentColor;

  border-radius: 2px;
  cursor: pointer;

  body.light & {
    background: white;
    color: black;
  }
  body.dark & {
    background: black;
    color: white;
  }

  &:hover {
    background: @colors-action-400 !important;
    color: white !important;
    border-color: @colors-action-400 !important;
  }

  .attributes-panel-item__section-header & {
    background: white;
    color: black;
    &:hover {
      background: @colors-action-400;
      color: white;
    }
  }
}
.attributes-panel-item__popout-edit-button,
.attributes-panel-item__popout-view-button {
  position: absolute;
  right: 4px;
  bottom: 4px;
  display: none;
  z-index: 51;
  transform: scaleX(-1);

  .attributes-panel-item.--input.--focus &,
  .attributes-panel-item.--input.--hover & {
    display: block;
  }
}

.attributes-panel-item__input-wrap:hover
  .attributes-panel-item__popout-view-button {
  display: block;
  z-index: 51;
}

.attributes-panel-item__add-child-row {
  height: 34px;

  .attributes-panel-item__nested-arrow-icon {
    margin-left: @indent-size;
  }
}
.attributes-panel-item__new-child-button {
  border-radius: 2px;
  padding: 4px 16px;
  display: flex;
  gap: 4px;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  margin-right: 8px;

  // unset a few shared styles from the other buttons
  width: unset;
  height: unset;
  background: none !important;

  .icon {
    width: 14px;
    height: 14px;
    margin-left: -2px;
  }
}

.attributes-panel-item.--input .attributes-panel-item__type-icon {
  opacity: 0.5;
}
.attributes-panel-item.--input.--invalid .attributes-panel-item__type-icon {
  color: @colors-destructive-500;
  opacity: 1;
}

.attributes-panel-item.--focus {
  .attributes-panel-item__input-wrap {
    border-color: var(--input-focus-border-color);
  }
}

// first input in a child list gets a bit of space
.attributes-panel-item.--input:first-child {
  margin-top: 8px;
}

// inputs next to each other push together to overlap their input borders
.attributes-panel-item.--input + .attributes-panel-item.--input {
  margin-top: -1px;
}

// add spacing when inputs/sections are next to each other
// and any sections after an open section
.attributes-panel-item.--section + .attributes-panel-item.--input,
.attributes-panel-item.--input + .attributes-panel-item.--section,
.attributes-panel-item.--section.--open + .attributes-panel-item.--section {
  margin-top: 8px;
}
</style>
