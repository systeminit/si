<template>
  <div
    class="attributes-panel-item"
    :class="{
      '--section': canHaveChildren,
      '--input': !canHaveChildren,
      '--hover': isHover,
      '--section-hover': isSectionHover,
      '--focus': isFocus,
      '--open': canHaveChildren && isOpen,
      '--collapsed': canHaveChildren && !isOpen,
      '--invalid': !isValid,
    }"
  >
    <div
      v-if="canHaveChildren"
      @mouseover.stop="onSectionHoverStart"
      @mouseleave="onSectionHoverEnd"
    >
      <div
        class="attributes-panel-item__section-header-wrap"
        :style="{
          top: topPx,
          zIndex: headerZIndex,
        }"
      >
        <div
          class="attributes-panel-item__section-toggle"
          @click="toggleOpen()"
        >
          <Icon
            :name="isOpen ? 'chevron--down' : 'chevron--right'"
            size="inherit"
          />
        </div>

        <div
          class="attributes-panel-item__section-header"
          :style="{ marginLeft: indentPx }"
          @click="toggleOpen(true)"
        >
          <Icon
            v-if="isChildOfMap || isChildOfArray"
            class="attributes-panel-item__nested-arrow-icon"
            name="nested-arrow-right"
            size="none"
          />
          <Icon
            class="attributes-panel-item__type-icon"
            :name="icon"
            size="none"
          />
          <div class="attributes-panel-item__section-header-label">
            <template v-if="isChildOfArray">
              {{ propName }}[{{ attributeDef.arrayIndex }}]
            </template>
            <template v-else-if="isChildOfMap">
              <span>{{ attributeDef.mapKey }}</span>
            </template>
            <template v-else>
              <span>{{ fullPropDef.name }}</span>
            </template>

            <span
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
            </span>
          </div>
          <div class="attributes-panel-item__action-icons">
            <!-- <Icon
              :name="isOpen ? 'collapse-row' : 'expand-row'"
              @click.stop="toggleOpen"
            /> -->
            <!-- <Icon v-if="isChildOfArray || isChildOfMap" name="trash" />
            <Icon v-if="isChildOfArray" name="arrow--up" />
            <Icon v-if="isChildOfArray" name="arrow--down" /> -->
          </div>
        </div>
      </div>

      <div
        v-show="isOpen"
        class="attributes-panel-item__left-border"
        :style="{ marginLeft: indentPx, zIndex: headerZIndex }"
      />

      <div v-show="isOpen" class="attributes-panel-item__children">
        <template v-if="attributeDef.children.length">
          <!-- <div class="w-[50%] h-[1px] bg-shade-0 ml-auto"></div> -->
          <AttributesPanelItem
            v-for="childProp in attributeDef.children"
            :key="`${propName}/${childProp.propDef?.name}`"
            :attributeDef="childProp"
            :level="level + 1"
          />
        </template>

        <div
          v-if="isArray || isMap"
          class="attributes-panel-item__add-child-row"
          :style="{ marginLeft: indentPx }"
        >
          <Icon
            class="attributes-panel-item__nested-arrow-icon"
            name="nested-arrow-right"
            size="none"
          />

          <input
            v-if="isMap"
            v-model="newMapChildKey"
            type="text"
            placeholder="key"
            class="attributes-panel-item__new-child-key-input"
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
      </div>
    </div>

    <div v-else class="attributes-panel-item__item-inner">
      <div
        class="attributes-panel-item__item-label"
        :style="{ paddingLeft: indentPx }"
      >
        <Icon
          v-if="isChildOfMap || isChildOfArray"
          class="attributes-panel-item__nested-arrow-icon"
          name="nested-arrow-right"
          size="none"
        />
        <div class="attributes-panel-item__item-label-text">
          <i>{{ propLabelParts[0] }}</i
          >{{ propLabelParts[1] }}
        </div>
        <button
          v-if="isChildOfMap || isChildOfArray"
          class="attributes-panel-item__delete-child-button hover:text-destructive-500"
          @click="removeChildHandler"
        >
          <Icon name="trash" size="none" />
        </button>
        <!-- TODO - enable tooltip help info -->
        <!-- <Icon
          v-if="propName === 'region'"
          v-tooltip="'Some help info'"
          name="question-circle"
          class="attributes-panel-item__help-icon"
        /> -->
        <a
          v-if="fullPropDef.docLink"
          class="attributes-panel-item__docs-icon"
          :href="fullPropDef.docLink"
          target="_blank"
          title="show docs"
        >
          <Icon class="attributes-panel-item__help-icon" name="docs" />
        </a>

        <div class="attributes-panel-item__action-icons">
          <!-- <Icon v-if="isChildOfArray || isChildOfMap" name="trash" size="sm" />
          <Icon v-if="isChildOfArray" name="arrow--up" size="sm" />
          <Icon v-if="isChildOfArray" name="arrow--down" size="sm" />
          <Icon v-if="isChildOfMap" name="edit" size="sm" /> -->
        </div>

        <Icon
          v-tooltip="attributeDef.validationError"
          :name="icon"
          size="sm"
          class="attributes-panel-item__type-icon"
        />
      </div>

      <div
        class="attributes-panel-item__input-wrap"
        @mouseover="onHoverStart"
        @mouseleave="onHoverEnd"
      >
        <Icon
          v-if="currentValue !== null"
          name="x-circle"
          class="attributes-panel-item__unset-button"
          @click="unsetHandler"
        />
        <template v-if="propKind === 'integer'">
          <input
            v-model="newValueNumber"
            type="number"
            spellcheck="false"
            @focus="onFocus"
            @blur="onBlur"
            @keyup.enter="updateValue"
          />
        </template>
        <template v-else-if="widgetKind === 'text'">
          <input
            v-model="newValueString"
            type="text"
            spellcheck="false"
            @focus="onFocus"
            @blur="onBlur"
            @keyup.enter="updateValue"
          />
        </template>
        <template v-else-if="widgetKind === 'password'">
          <!-- todo add show/hide controls -->
          <input
            v-model="newValueString"
            type="password"
            @focus="onFocus"
            @blur="onBlur"
            @keyup.enter="updateValue"
          />
        </template>
        <template
          v-else-if="widgetKind === 'textArea' || widgetKind === 'codeEditor'"
        >
          <textarea
            v-model="newValueString"
            spellcheck="false"
            @focus="onFocus"
            @blur="onBlur"
            @keydown.enter="(e) => e.metaKey && updateValue()"
          />
          <Icon
            name="external-link"
            class="attributes-panel-item__popout-edit-button"
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
            type="checkbox"
            class="attributes-panel-item__hidden-input"
            @input="(e) => newValueBoolean = (e.target as HTMLInputElement)?.checked"
            @focus="onFocus"
            @blur="onBlur"
            @change="updateValue"
          />
          <div class="attributes-panel-item__input-value">
            <Icon
              class="attributes-panel-item__checkbox-icon"
              :name="newValueBoolean === true ? 'check-square' : 'empty-square'"
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
            @focus="onFocus"
            @blur="onBlur"
            @change="updateValue"
          >
            <option v-for="o in widgetOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </option>
          </select>
          <div class="attributes-panel-item__input-value">
            {{ currentValue }}
          </div>
          <Icon
            name="input-type-select"
            class="absolute right-1 top-1 text-neutral-400 dark:text-neutral-600"
            size="sm"
          />
        </template>
        <template v-else-if="widgetKind === 'secret'">
          <div
            class="attributes-panel-item__secret-value-wrap"
            @click="secretPopoverRef?.open($event)"
          >
            <div v-if="secret" class="attributes-panel-item__secret-value">
              <Icon name="key" size="xs" />
              {{ secretDefinitionId }} / {{ secret.name }}
            </div>
            <div v-else class="attributes-panel-item__secret-value-empty">
              select/add secret
            </div>
          </div>

          <Popover
            ref="secretPopoverRef"
            anchorDirectionX="left"
            anchorAlignY="bottom"
          >
            <SecretsPopover
              v-if="secretDefinitionId"
              :definitionId="secretDefinitionId"
              @select="secretSelectedHandler"
            />
          </Popover>
        </template>
        <template v-else>
          <div class="py-[4px] px-[8px] text-sm">
            {{ widgetKind }}
          </div>
        </template>
      </div>
      <!-- <Icon name="none" class="p-[3px] mx-[2px]" /> -->
    </div>

    <Modal
      v-if="widgetKind === 'textArea' || widgetKind === 'codeEditor'"
      ref="editModalRef"
      size="4xl"
      :title="`Edit value - ${propLabel}`"
      class="attributes-panel-item__edit-value-modal"
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
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { computed, PropType, ref, watch } from "vue";

import { Icon, IconNames, Modal } from "@si/vue-lib/design-system";
import {
  AttributeTreeItem,
  useComponentAttributesStore,
} from "@/store/component_attributes.store";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Secret, useSecretsStore } from "@/store/secrets.store";
import AttributesPanelItem from "./AttributesPanelItem.vue"; // eslint-disable-line import/no-self-import
import { useAttributesPanelContext } from "./AttributesPanel.vue";
import CodeEditor from "../CodeEditor.vue";
import Popover from "../Popover.vue";
import SecretsPopover from "../SecretsPopover.vue";

const props = defineProps({
  parentPath: { type: String },
  attributeDef: { type: Object as PropType<AttributeTreeItem>, required: true },
  level: { type: Number, default: 0 },
  isArrayItem: Boolean,
  startCollapsed: { type: Boolean },
  // number of prop keys to show while collapsed
  numPreviewProps: { type: Number, default: 3 },
});

const isOpen = ref(true);

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
const isChildOfArray = computed(
  () => props.attributeDef.arrayIndex !== undefined,
);
const isChildOfMap = computed(() => props.attributeDef.mapKey !== undefined);

const isValid = computed(() => props.attributeDef.isValid);

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

function resetNewValueToCurrentValue() {
  newValueBoolean.value = !!currentValue.value;
  newValueString.value = currentValue.value?.toString() || "";
  const valAsNumber = parseFloat(currentValue.value?.toString() || "");
  newValueNumber.value = Number.isNaN(valAsNumber) ? undefined : valAsNumber;
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

function getKey() {
  if (isChildOfMap.value) return props.attributeDef?.mapKey;

  return props.attributeDef?.arrayKey;
}

function addChildHandler() {
  const isAddingMapChild = propKind.value === "map";
  if (isAddingMapChild && !newMapChildKeyIsValid.value) return;

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
  // TODO: figure out number handling
  // also clarify how we want to handle null/undefined versus empty string
  newValueBoolean.value = false;
  newValueString.value = "";
  attributesStore.UPDATE_PROPERTY_VALUE({
    update: {
      attributeValueId: props.attributeDef.valueId,
      parentAttributeValueId: props.attributeDef.parentValueId,
      propId: props.attributeDef.propId,
      componentId,
      value: null,
    },
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

  // dont trigger an update if the value has not changed
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
  isHover.value = true;
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

const editModalRef = ref<InstanceType<typeof Modal>>();

const secretPopoverRef = ref<InstanceType<typeof Popover>>();
const secretsStore = useSecretsStore();
const secret = computed(
  () => secretsStore.secretsById[newValueString.value?.toString() || ""],
);
const secretDefinitionId = computed(() => {
  if (props.attributeDef.propDef.widgetKind.kind !== "secret") return;
  const widgetOptions = props.attributeDef.propDef.widgetKind.options;
  // WHAT? this setup doesn't really make sense...
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
  secretPopoverRef.value?.close();
}
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
  align-items: center;
  border-bottom: 1px solid var(--panel-bg-color);
  user-select: none;
}

.attributes-panel-item__section-header-child-count {
  font-style: italic;
  margin-left: 12px;
  font-size: 12px;
  opacity: 0.5;
}

.attributes-panel-item__section-toggle {
  cursor: pointer;
  position: absolute;
  width: @header-height;
  height: @header-height;
  opacity: 0.8;
  transition: all 0.2s;

  body.light & {
    color: @colors-neutral-700;
  }
  body.dark & {
    color: @colors-white;
  }

  &:hover .icon {
    transform: scale(1.1);
  }

  .attributes-panel.--show-section-toggles & {
    opacity: 1;
  }
}

.attributes-panel-item__nested-arrow-icon {
  width: 14px;
  height: 14px;
}
.attributes-panel-item__type-icon {
  height: 100%;
  padding: 2px;
  margin-right: @spacing-px[xs];
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
  flex-shrink: 1;
  overflow: hidden;
  text-overflow: ellipsis;
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
    padding-right: 28px; // to give space for unset button
    width: 100%;
    border: none;
    font-size: inherit;
    line-height: inherit;
    display: block;
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
}
.attributes-panel-item__input-value {
  padding: 5px 8px;
  display: flex;
  align-items: center;
}

.attributes-panel-item__action-icons {
  gap: 4px;
  padding: 4px;
  margin-left: 10px;
  margin-right: 10px;

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
    margin-left: 30px;
  }
  .attributes-panel-item__section-header:hover & {
    display: flex;
  }
}

// small icon buttons
.attributes-panel-item__action-icons .icon,
.attributes-panel-item__popout-edit-button,
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
.attributes-panel-item__popout-edit-button {
  position: absolute;
  right: 4px;
  bottom: 4px;
  display: none;
  z-index: 102;
  transform: scaleX(-1);

  .attributes-panel-item.--input.--focus &,
  .attributes-panel-item.--input.--hover & {
    display: block;
  }
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

.attributes-panel-item.--input .attributes-panel-item__type-icon {
  margin-left: auto;
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
  color: white;
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
  padding-left: 4px;
  cursor: pointer;
  // text-align: center;
  &:hover {
    opacity: 1;
  }
}

.attributes-panel-item__edit-value-modal {
  textarea {
    @apply focus:ring-0 focus:ring-offset-0;
    background: transparent;
    width: 100%;
    height: 100%;
    overflow: auto;
    position: absolute;
    border: none;
    font-size: 14px;
    line-height: 20px;
    font-family: monospace;
    resize: none;
    display: block;
  }
}
.attributes-panel-item__edit-value-modal-code-wrap {
  height: 40vh;
  position: relative;
  border: 1px solid var(--input-focus-border-color);
  background: var(--input-focus-bg-color);
  // margin-bottom: @spacing-px[xs];
}
</style>
