<template>
  <div
    :class="{
      '--section': canHaveChildren,
      '--input': !canHaveChildren,
      '--hover': isHover,
      '--section-hover': isSectionHover,
      '--focus': isFocus,
      '--open': canHaveChildren && isOpen,
      '--collapsed': canHaveChildren && !isOpen,
    }"
    class="attributes-panel-item"
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
        class="attributes-panel-item__section-header-wrap"
      >
        <div
          :class="
            clsx(
              'attributes-panel-item__section-toggle',
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
          />
        </div>

        <div
          :style="{ marginLeft: indentPx }"
          class="attributes-panel-item__section-header"
          @click="toggleOpen(true)"
        >
          <Icon
            v-if="isChildOfMap || isChildOfArray"
            class="attributes-panel-item__nested-arrow-icon"
            name="nested-arrow-right"
            size="none"
          />
          <Icon
            :name="icon"
            class="attributes-panel-item__type-icon"
            size="none"
          />
          <div class="attributes-panel-item__section-header-label">
            <div
              ref="headerMainLabelRef"
              v-tooltip="headerMainLabelTooltip"
              class="attributes-panel-item__section-header-label-main leading-loose"
            >
              <template v-if="isChildOfArray">
                {{ propName }}[{{ attributeDef.arrayIndex }}]
              </template>
              <template v-else-if="isChildOfMap">
                {{ attributeDef.mapKey }}
              </template>
              <template v-else>
                {{ fullPropDef.name }}
              </template>
            </div>

            <div
              v-if="isMap || isArray"
              class="attributes-panel-item__section-header-child-count"
            >
              <template v-if="attributeDef.children.length === 0"
                >(empty)</template
              >
              <template v-else-if="attributeDef.children.length === 1"
                >(1 item)</template
              >
              <template v-else
                >({{ attributeDef.children.length }} items)</template
              >
            </div>
          </div>
          <SourceIconWithTooltip
            v-if="
              featureFlagsStore.INDICATORS_MANUAL_FUNCTION_SOCKET &&
              !(widgetKind === 'secret')
            "
            :icon="sourceIcon"
            :overridden="sourceOverridden"
            :tooltipText="sourceTooltip"
            header
          />
          <!-- DROPDOWN MENU FOR SELECT SOURCE -->
          <template
            v-if="
              validAttributeValueSources.length > 1 &&
              featureFlagsStore.INDICATORS_MANUAL_FUNCTION_SOCKET
            "
          >
            <div
              class="attributes-panel-item__section-header-source-select"
              @click="sourceSelectMenuRef?.open($event)"
            >
              <div>set:</div>
              <div
                class="flex flex-row items-center border pl-2xs pr-[2px] h-4 text-xs"
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
        class="attributes-panel-item__left-border"
      />

      <!-- CHILDREN -->
      <div
        v-show="isOpen && headerHasContent"
        class="attributes-panel-item__children"
      >
        <template v-if="attributeDef.children.length">
          <!-- <div class="w-[50%] h-[1px] bg-shade-0 ml-auto"></div> -->
          <AttributesPanelItem
            v-for="childProp in attributeDef.children"
            :key="`${propName}/${childProp.propDef?.name}`"
            :attributeDef="childProp"
            :level="level + 1"
          />
        </template>
        <!-- <div
          v-else
          :style="{
            marginLeft: `${HEADER_HEIGHT + INDENT_SIZE * (props.level + 1)}px`,
          }"
        >
          This prop does not currently have any children.
        </div> -->

        <template v-if="(isArray || isMap) && propManual">
          <div
            :style="{ marginLeft: indentPx }"
            class="attributes-panel-item__add-child-row"
          >
            <Icon
              class="attributes-panel-item__nested-arrow-icon"
              name="nested-arrow-right"
              size="none"
            />

            <input
              v-if="isMap"
              v-model="newMapChildKey"
              :class="
                clsx(
                  'attributes-panel-item__new-child-key-input',
                  isMapKeyError &&
                    'attributes-panel-item__new-child-key-input__error',
                )
              "
              placeholder="key"
              type="text"
              @blur="clearKeyError"
              @keyup.enter="addChildHandler"
            />

            <button
              class="attributes-panel-item__new-child-button"
              @click="addChildHandler"
            >
              <Icon name="plus" size="none" />
              {{ isArray ? "Add array item" : "Add map item" }}
            </button>
          </div>

          <div
            v-if="isMap && isMapKeyError"
            :style="{ marginLeft: indentPx }"
            class="attributes-panel-item__map-key-error"
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
      class="attributes-panel-item__item-inner"
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
          class="attributes-panel-item__nested-arrow-icon"
          name="nested-arrow-right"
          size="none"
        />
        <div
          :title="`${propLabelParts[0]}${propLabelParts[1]}`"
          class="attributes-panel-item__item-label-text"
        >
          <template v-if="isChildOfMap">{{ propLabelParts[1] }}</template>
          <template v-else-if="isChildOfArray">
            [{{ props.attributeDef.arrayIndex }}]
          </template>
          <template v-else>{{ propLabel }}</template>
        </div>

        <!-- TODO - enable tooltip help info -->
        <!-- <Icon
          v-if="propName === 'region'"
          v-tooltip="'Some help info'"
          name="question-circle"
          class="attributes-panel-item__help-icon"
        /> -->

        <div class="attributes-panel-item__static-icons">
          <button
            v-if="isChildOfMap || isChildOfArray"
            class="attributes-panel-item__delete-child-button hover:scale-125"
            @click="removeChildHandler"
          >
            <Icon name="trash" size="xs" />
          </button>

          <SourceIconWithTooltip
            v-if="
              featureFlagsStore.INDICATORS_MANUAL_FUNCTION_SOCKET &&
              !(widgetKind === 'secret')
            "
            :icon="sourceIcon"
            :overridden="sourceOverridden"
            :tooltipText="sourceTooltip"
          />

          <a
            v-if="fullPropDef.docLink"
            :href="fullPropDef.docLink"
            class="attributes-panel-item__docs-icon hover:scale-125"
            target="_blank"
            title="show docs"
          >
            <Icon class="attributes-panel-item__help-icon" name="docs" />
          </a>
        </div>
      </div>

      <div
        :class="{
          'force-border-red-400': validation && validation.status !== 'Success',
          'my-1': validation && validation.status !== 'Success',
        }"
        class="attributes-panel-item__input-wrap"
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
          v-if="
            sourceOverridden &&
            currentValue !== null &&
            !propPopulatedBySocket &&
            !propControlledByParent
          "
          class="attributes-panel-item__unset-button"
          name="x-circle"
          @click="unsetHandler"
        />
        <template v-if="propKind === 'integer'">
          <input
            v-model="newValueNumber"
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
            class="attributes-panel-item__hidden-input"
            type="checkbox"
            @blur="onBlur"
            @change="updateValue"
            @focus="onFocus"
            @input="(e) => newValueBoolean = (e.target as HTMLInputElement)?.checked"
          />
          <div class="attributes-panel-item__input-value">
            <Icon
              :name="newValueBoolean === true ? 'check-square' : 'empty-square'"
              class="attributes-panel-item__checkbox-icon"
            />
            {{ newValueBoolean ? "TRUE" : "FALSE" }}
          </div>
        </template>
        <template
          v-else-if="widgetKind === 'comboBox' || widgetKind === 'select'"
        >
          <select
            v-model="newValueString"
            class="attributes-panel-item__hidden-input"
            @blur="onBlur"
            @change="updateValue"
            @focus="onFocus"
          >
            <option v-for="o in widgetOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </option>
          </select>
          <div class="attributes-panel-item__input-value">
            {{ currentValue }}
          </div>
          <Icon
            class="absolute right-1 top-1 text-neutral-400 dark:text-neutral-600"
            name="input-type-select"
            size="sm"
          />
        </template>
        <template v-else-if="widgetKind === 'secret'">
          <div
            class="attributes-panel-item__secret-value-wrap"
            @click="secretModalRef?.open()"
          >
            <div v-if="secret" class="attributes-panel-item__secret-value">
              <Icon name="key" size="xs" />
              {{ secretDefinitionId }} / {{ secret.name }}
            </div>
            <div v-else class="attributes-panel-item__secret-value-empty">
              select/add secret
            </div>
          </div>

          <SecretsModal
            v-if="secretDefinitionId"
            ref="secretModalRef"
            :definitionId="secretDefinitionId"
            @select="secretSelectedHandler"
          />
        </template>
        <template v-else>
          <div class="py-[4px] px-[8px] text-sm">{{ widgetKind }}</div>
        </template>
        <div
          v-if="
            propControlledByParent ||
            (featureFlagsStore.INDICATORS_MANUAL_FUNCTION_SOCKET &&
              propSetByDynamicFunc &&
              !editOverride)
          "
          v-tooltip="
            propControlledByParent
              ? `${propName} is set via a function from an ancestor`
              : `${propName} is set via an input socket`
          "
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

      <Icon
        v-if="validation?.status === 'Success'"
        class="mr-2"
        name="check"
        tone="success"
      />
      <Icon v-else-if="validation" class="mr-2" name="x" tone="error" />
    </div>

    <!-- VALIDATION DETAILS -->
    <div
      v-if="showValidationDetails && validation"
      :style="{ marginLeft: indentPx }"
      class="text-red-400 flex flex-col bg-black pl-3 border-y border-red-400 pb-1 my-1"
    >
      <p class="my-3">{{ validation.message }}</p>

      <span
        v-for="(output, index) in validation.logs"
        :key="index"
        class="text-sm break-all text-warning-500"
      >
        <p v-if="output.stream !== 'output'">{{ output.message }}</p>
      </span>
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
      <div class="attributes-panel-item__edit-value-modal-code-wrap">
        <template v-if="widgetKind === 'textArea'">
          <textarea v-model="newValueString" spellcheck="false" />
        </template>
        <template v-else-if="widgetKind === 'codeEditor'">
          <CodeEditor
            :id="`${changeSetsStore.selectedChangeSetId}/${attributeDef.valueId}`"
            v-model="newValueString"
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
      <div class="attributes-panel-item__view-value-modal-code-wrap">
        <template v-if="widgetKind === 'textArea'">
          <pre class="attributes-panel-item__edit-value-modal__view-text">
          {{ newValueString }}
          </pre>
        </template>
        <template v-else-if="widgetKind === 'codeEditor'">
          <CodeViewer :code="newValueString" />
        </template>
      </div>
    </Modal>

    <!-- MODAL FOR WHEN YOU CLICK A PROP WHICH IS CONTROLLED BY A PARENT OR SOCKET -->
    <Modal
      ref="confirmEditModalRef"
      :title="
        propControlledByParent
          ? `You Cannot Edit Prop &quot;${propName}&quot;`
          : 'Are You Sure?'
      "
    >
      <div class="pb-sm">
        <template v-if="propControlledByParent">
          You cannot edit prop "{{ propName }}" because it is populated by a
          function from an ancestor prop.
        </template>
        <template v-else>
          Editing the prop "{{ propName }}" directly will override the value
          that is set by a dynamic function.
        </template>
      </div>
      <div class="flex gap-sm">
        <VButton
          :class="propControlledByParent ? 'flex-grow' : ''"
          icon="x"
          tone="shade"
          variant="ghost"
          @click="closeConfirmEditModal"
        >
          Cancel
        </VButton>
        <VButton
          v-if="!propControlledByParent"
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
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import AttributesPanelItem from "./AttributesPanelItem.vue"; // eslint-disable-line import/no-self-import
import { useAttributesPanelContext } from "./AttributesPanel.vue";
import CodeEditor from "../CodeEditor.vue";
import SecretsModal from "../SecretsModal.vue";
import SourceIconWithTooltip from "./SourceIconWithTooltip.vue";
import CodeViewer from "../CodeViewer.vue";

const props = defineProps({
  parentPath: { type: String },
  attributeDef: { type: Object as PropType<AttributeTreeItem>, required: true },
  level: { type: Number, default: 0 },
  isArrayItem: Boolean,
  startCollapsed: { type: Boolean },
  // number of prop keys to show while collapsed
  numPreviewProps: { type: Number, default: 3 },
});

const featureFlagsStore = useFeatureFlagsStore();

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

const headerHasContent = computed(() => {
  return (
    props.attributeDef.children.length ||
    ((isArray.value || isMap.value) && propManual.value)
  );
});

const rootCtx = useAttributesPanelContext();

// not reactive - and we know it's populated - since the parent will rerender if it changes
const componentsStore = useComponentsStore();
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const componentId = componentsStore.selectedComponentId!;

const changeSetsStore = useChangeSetsStore();

const attributesStore = useComponentAttributesStore(componentId);

// const path = computed(() => {
//   if (!props.parentPath) return props.prop?.toString() || "";
//   return `${props.parentPath}.${props.prop}`;
// });

const fullPropDef = computed(() => props.attributeDef.propDef);
const propKind = computed(() => fullPropDef.value.kind);
const widgetKind = computed(() => fullPropDef.value.widgetKind.kind);
const widgetOptions = computed(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  () => (fullPropDef.value.widgetKind as any).options,
);
const propName = computed(() => fullPropDef.value.name);
const propLabelParts = computed(() => {
  if (isChildOfArray.value)
    return [`${propName.value}[${props.attributeDef.arrayIndex}]`];
  if (isChildOfMap.value)
    return [`${propName.value}.`, props.attributeDef.mapKey];
  return ["", propName.value];
});
const propLabel = computed(() => propLabelParts.value.join(""));

const isArray = computed(() => propKind.value === "array");
const isMap = computed(() => propKind.value === "map");
const isMapKeyError = ref(false);
const clearKeyError = () => {
  isMapKeyError.value = false;
};
const isChildOfArray = computed(
  () => props.attributeDef.arrayIndex !== undefined,
);
const isChildOfMap = computed(() => props.attributeDef.mapKey !== undefined);

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
  if (propKind.value === "array") return "brackets-square";
  if (propKind.value === "map") return "brackets-curly";
  if (propKind.value === "object") return "bullet-list";
  if (propKind.value === "integer") return "input-type-number";
  return WIDGET_ICON_LOOKUP[widgetKind.value] || "question-circle";
});

const HEADER_HEIGHT = 24;
const INDENT_SIZE = 8;

const indentPx = computed(
  () => `${HEADER_HEIGHT + INDENT_SIZE * props.level}px`,
);
const topPx = computed(() => `${HEADER_HEIGHT * props.level}px`);

const headerZIndex = computed(() => 300 - props.level);

const newMapChildKey = ref("");

const currentValue = computed(() => props.attributeDef.value?.value);
const newValueBoolean = ref<boolean>();
const newValueString = ref<string>("");
const newValueNumber = ref<number>();
const noValue = computed(
  () => currentValue.value === null && newValueString.value === "",
);
const iconShouldBeHidden = computed(
  () => icon.value === "input-type-select" || icon.value === "check",
);

const propPopulatedBySocket = computed(
  () => props.attributeDef.value?.isFromExternalSource,
);
const propHasSocket = computed(
  () => props.attributeDef.value?.canBeSetBySocket,
);
const propSetByDynamicFunc = computed(
  () =>
    props.attributeDef.value?.isControlledByDynamicFunc &&
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

  if (props.attributeDef.propDef.defaultCanBeSetBySocket) {
    sources.push(AttributeValueSource.Socket);
  }

  if (props.attributeDef.value?.isControlledByAncestor === false) {
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
    const value = props.attributeDef.value?.value ?? null;

    attributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: props.attributeDef.valueId,
        parentAttributeValueId: props.attributeDef.parentValueId,
        propId: props.attributeDef.propId,
        componentId,
        value,
      },
    });
  } else {
    attributesStore.RESET_PROPERTY_VALUE({
      attributeValueId: props.attributeDef.valueId,
    });
  }
};

const sourceIcon = computed(() => {
  if (propPopulatedBySocket.value) return "circle-full";
  else if (propSetByDynamicFunc.value) return "func";
  else if (propHasSocket.value) return "circle-empty";
  else return "cursor";
});

const sourceOverridden = computed(() => props.attributeDef.value?.overridden);

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
    } else if (propSetByDynamicFunc.value) {
      return `${propName.value} is set by a dynamic function`;
    } else if (propHasSocket.value) {
      return `${propName.value} is set via an empty socket`;
    }
    return `${propName.value} can be set manually`;
  }
});

const propControlledByParent = computed(
  () => props.attributeDef.value?.isControlledByAncestor,
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

  attributesStore.REMOVE_PROPERTY_VALUE({
    attributeValueId: props.attributeDef.valueId,
    propId: props.attributeDef.propId,
    componentId,
    key: getKey(),
  });
}

const validation = computed(
  () =>
    (_.find(
      props.attributeDef?.validations,
      ([key]) => key === (getKey() ?? null),
    ) ?? [])[1],
);

function getKey() {
  if (isChildOfMap.value) return props.attributeDef?.mapKey;

  return props.attributeDef?.arrayKey;
}

function addChildHandler() {
  const isAddingMapChild = propKind.value === "map";
  if (isAddingMapChild && !newMapChildKeyIsValid.value) {
    isMapKeyError.value = true;
    return;
  }

  attributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: props.attributeDef.valueId,
      propId: props.attributeDef.propId,
      componentId,
      ...(isAddingMapChild && {
        key: newMapChildKey.value.trim(),
      }),
    },
  });
  newMapChildKey.value = "";
}
function unsetHandler() {
  newValueBoolean.value = false;
  newValueString.value = "";

  attributesStore.RESET_PROPERTY_VALUE({
    attributeValueId: props.attributeDef.valueId,
  });
}
function updateValue() {
  let newVal;
  let skipUpdate = false;
  if (widgetKind.value === "checkbox") {
    newVal = newValueBoolean.value;
    // special handling for empty value + false
    if (newVal === false && !currentValue.value) skipUpdate = true;
  } else if (propKind.value === "integer") {
    // There is no such thing as an integer widget kind!
    newVal = newValueNumber.value;
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

  attributesStore.UPDATE_PROPERTY_VALUE({
    update: {
      attributeValueId: props.attributeDef.valueId,
      parentAttributeValueId: props.attributeDef.parentValueId,
      propId: props.attributeDef.propId,
      componentId,
      value: newVal,
    },
  });
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
  rootCtx.hoverSectionValueId.value = props.attributeDef.valueId;
}
function onSectionHoverEnd() {
  isHover.value = false;
  if (rootCtx.hoverSectionValueId.value === props.attributeDef.valueId) {
    rootCtx.hoverSectionValueId.value = undefined;
  }
}
const isSectionHover = computed(
  () => rootCtx.hoverSectionValueId.value === props.attributeDef.valueId,
);

const viewModalRef = ref<InstanceType<typeof Modal>>();
const editModalRef = ref<InstanceType<typeof Modal>>();
const secretModalRef = ref<InstanceType<typeof SecretsModal>>();
const secretsStore = useSecretsStore();
const secret = computed(
  () => secretsStore.secretsById[newValueString.value?.toString() || ""],
);
const secretDefinitionId = computed(() => {
  if (props.attributeDef.propDef.widgetKind.kind !== "secret") return;
  const widgetOptions = props.attributeDef.propDef.widgetKind.options;
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

.attributes-panel-item {
  position: relative;
  font-size: 14px;

  body.light & {
    --header-bg-color: @colors-neutral-500;
    --header-text-color: @colors-white;
    &.--section-hover {
      --header-bg-color: @colors-neutral-900;
      --header-text-color: @colors-white;
    }
  }
  body.dark & {
    --header-bg-color: @colors-neutral-600;
    --header-text-color: @colors-white;
    &.--section-hover {
      --header-bg-color: @colors-neutral-300;
      --header-text-color: @colors-black;
    }
  }
}

.attributes-panel-item__section-header-wrap {
  position: sticky;
  height: @header-height;
}

.attributes-panel-item__section-header {
  height: inherit;
  background: var(--header-bg-color);
  color: var(--header-text-color);
  display: flex;
  flex-direction: row;
  gap: 4px;
  align-items: center;
  border-bottom: 1px solid var(--panel-bg-color);
  user-select: none;
  padding-right: 4px;
}

.attributes-panel-item__section-header-label-main {
  flex-basis: 0px;
  flex-grow: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.attributes-panel-item__section-header-child-count {
  font-style: italic;
  margin-right: 4px;
  font-size: 12px;
  opacity: 0.5;
  flex: none;
}

.attributes-panel-item__section-toggle {
  background-color: var(--toggle-controls-bg-color);
  position: absolute;
  width: @header-height;
  height: @header-height;
  transition: all 0.2s;

  body.light & {
    color: @colors-neutral-700;
  }
  body.dark & {
    color: @colors-white;
  }

  .icon {
    opacity: 0.8;
  }

  &:hover .icon {
    transform: scale(1.1);
    opacity: 1;
  }

  .attributes-panel.--show-section-toggles & {
    opacity: 1;
  }
}

.attributes-panel-item__section-header-label {
  display: flex;
  flex-direction: row;
  align-items: center;
  white-space: nowrap;
  gap: 4px;
  min-width: 0;
}

.attributes-panel-item__nested-arrow-icon {
  width: 14px;
  height: 14px;
}
.attributes-panel-item__type-icon {
  height: 100%;
  padding: 2px;
  position: relative;
}

.attributes-panel-item__left-border {
  background: var(--header-bg-color);
  position: absolute;
  width: 1px;
  top: 0;
  bottom: 0;
  pointer-events: none;
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
.attributes-panel-item__checkbox-icon {
  display: inline-block;
  width: 22px;
  height: 22px;
  margin: -4px 0;
  margin-left: -4px;
  margin-right: 4px;
  padding: 0;
}

.attributes-panel-item__item-inner {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
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

.attributes-panel-item__help-icon {
  color: @colors-neutral-400;
  padding: 3px;
  cursor: pointer;
  &:hover {
    color: white;
  }
}

.force-border-red-400 {
  border-color: rgb(251 113 133 / var(--tw-border-opacity)) !important;
}
.attributes-panel-item__input-wrap {
  position: relative;
  border: 1px solid var(--input-border-color);
  min-height: 30px;
  width: 45%;
  flex-shrink: 0;
  background: var(--input-bg-color);
  margin-right: 8px;
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
.attributes-panel-item__input-value {
  padding: 5px 8px;
  display: flex;
  align-items: center;
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
  z-index: 49;
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
.attributes-panel-item__new-child-key-input {
  @apply focus:ring-0 focus:ring-offset-0;
  border: 1px solid var(--input-border-color);
  background: var(--input-bg-color);
  padding: 4px;
  height: 28px;
  font-size: inherit;
  flex-shrink: 1;
  min-width: 80px;
  width: 150px;

  &:focus {
    border-color: var(--input-focus-border-color);
    background: var(--input-focus-bg-color);
  }
}

.attributes-panel-item__new-child-key-input__error {
  border-color: #ef4444;

  &:focus {
    border-color: #ef4444;
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

.attributes-panel-item__delete-child-button {
  z-index: 30;
  flex: none;
  &:hover {
    color: @colors-destructive-500;
    opacity: 1;
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

.attributes-panel-item__unset-button {
  position: absolute;
  right: 0px;
  top: 0px;
  width: 28px;
  height: 28px;
  padding: 3px;
  opacity: 0.5;
  cursor: pointer;
  display: none;
  z-index: 2;
  &:hover {
    opacity: 1;
  }

  .attributes-panel-item.--input.--hover &,
  .attributes-panel-item.--input.--focus & {
    display: block;
  }
}

// SECRETS
.attributes-panel-item__secret-value-wrap {
  padding: 4px;
}
.attributes-panel-item__secret-value {
  background: @colors-action-700;
  display: inline-block;
  padding: 2px 10px;
  border-radius: 4px;
  line-height: 18px;
  font-size: 13px;
  cursor: pointer;

  .icon {
    height: 12px;
    width: 12px;
    display: inline-block;
    vertical-align: middle;
  }
}
.attributes-panel-item__secret-value-empty {
  opacity: 0.6;
  font-style: italic;
  padding-left: 24px;
  cursor: pointer;
  // text-align: center;
  &:hover {
    opacity: 1;
  }
}

.attributes-panel-item__edit-value-modal,
.attributes-panel-item__view-value-modal {
  textarea {
    @apply focus:ring-0 focus:ring-offset-0;
    border: none;
  }

  .attributes-panel-item__edit-value-modal__view-text {
    padding: 0.5rem;
    border: 1px solid var(--input-border-color);
  }

  textarea,
  .attributes-panel-item__edit-value-modal__view-text {
    background: transparent;
    width: 100%;
    height: 100%;
    overflow: auto;
    position: absolute;
    font-size: 14px;
    line-height: 20px;
    font-family: monospace;
    resize: none;
    display: block;
  }
}
.attributes-panel-item__edit-value-modal-code-wrap,
.attributes-panel-item__view-value-modal-code-wrap {
  height: 40vh;
  position: relative;
  background: var(--input-focus-bg-color);
}
.attributes-panel-item__edit-value-modal-code-wrap {
  border: 1px solid var(--input-focus-border-color);
}
.attributes-panel-item__view-value-modal-code-wrap {
  border: 1px solid var(--input-border-color);
}

.attributes-panel-item__map-key-error {
  padding-left: 2rem;
  padding-bottom: 0.5rem;
  font-style: italic;
  color: @colors-destructive-500;
}

.attributes-panel-item__static-icons {
  display: flex;
  flex-direction: row;
  margin-left: auto;
  align-items: center;
  flex: none;
  gap: 0.25rem;
  margin-right: 0.25rem;
}

.attributes-panel-item__static-icons > * {
  cursor: pointer;
}
</style>
