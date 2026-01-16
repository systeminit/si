<template>
  <div class="json-tree-explorer__row">
    <div
      class="json-tree-explorer__row-inner"
      @click.left="toggleOpen()"
      @click.right.prevent="rootCtx.openContextMenu($event, path, type)"
    >
      <!-- <Icon
        v-if="!['object', 'array'].includes(type)"
        class="json-tree-explorer__extract-button"
        name="check"
        title="Extract to field"
      /> -->
      <div
        class="json-tree-explorer__prop-name"
        :style="{ paddingLeft: indentPx }"
        :class="{ '--array-item': isArrayItem }"
      >
        {{ prop }}
      </div>
      <template v-if="canHaveChildren">
        <Icon
          v-if="!isEmpty"
          class="json-tree-explorer__collapse-toggle"
          :name="!isOpen ? 'chevron--right' : 'chevron--down'"
        />
        <div class="json-tree-explorer__bracket --left">{{ leftBracket }}</div>
        <template v-if="!isOpen || isEmpty">
          <div
            class="json-tree-explorer__children_summary"
            :class="{ '--empty': isEmpty }"
          >
            {{ childSummary }}
          </div>
          <div class="json-tree-explorer__bracket --right">
            {{ rightBracket }}
          </div>
        </template>
      </template>
      <component
        :is="valueUrl ? 'a' : 'div'"
        v-else
        class="json-tree-explorer__value"
        :href="valueUrl ? valueUrl : undefined"
        :target="valueUrl ? '_blank' : undefined"
        :class="valueClasses"
        >{{ stringValue }}</component
      >
    </div>
    <template v-if="canHaveChildren && !isEmpty">
      <div v-show="isOpen" class="json-tree-explorer__children">
        <div
          class="json-tree-explorer__children-border-left"
          :style="{ marginLeft: indentPx }"
        ></div>
        <JsonTreeExplorerItem
          v-for="(v, key) in value"
          :key="`${path}/${key}`"
          :parentPath="path?.toString()"
          :prop="key"
          :value="v"
          :isArrayItem="type === 'array'"
          :level="level + 1"
        />
      </div>
      <div
        v-if="isOpen"
        class="json-tree-explorer__bracket --right --newline"
        :style="{ paddingLeft: indentPx }"
      >
        {{ rightBracket }}
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-explicit-any */

import * as _ from "lodash-es";
import { computed, onMounted, ref } from "vue";
import { Icon } from "../..";

import JsonTreeExplorerItem from "./JsonTreeExplorerItem.vue"; // eslint-disable-line import/no-self-import
import { useJsonTreeRootContext } from "./JsonTreeExplorer.vue";

const props = defineProps({
  parentPath: { type: String },
  prop: [String, Number],
  // eslint-disable-next-line vue/require-prop-types
  value: {}, // any
  level: { type: Number, default: 0 },
  isArrayItem: Boolean,
  startCollapsed: { type: Boolean },
  // number of prop keys to show while collapsed
  numPreviewProps: { type: Number, default: 3 },
});

const isOpen = ref(false);

const rootCtx = useJsonTreeRootContext();

const path = computed(() => {
  if (!props.parentPath) return props.prop?.toString() || "";
  return `${props.parentPath}.${props.prop}`;
});
const type = computed(() => {
  if (_.isString(props.value)) {
    return valueUrl.value ? "link" : "string";
  }
  if (_.isNumber(props.value)) return "number";
  if (_.isArray(props.value)) return "array";
  if (_.isObject(props.value)) return "object";
  if (_.isBoolean(props.value)) return "boolean";
  if (_.isNil(props.value)) return "empty";
  return "unknown";
});
const canHaveChildren = computed(() => {
  return ["object", "array"].includes(type.value);
});

const valueUrl = computed(() => {
  if (!_.isString(props.value)) return null;
  if ((props.value).match(/https?:\/\//)) {
    // TODO: smarter handling - add http if it looks like a url without it
    return props.value;
    // } else if (this.value.match(EMAIL_REGEX)) {
    //   return `mailto:${this.value}`;
  }
  return null;
});

const valueClasses = computed(() => {
  return {
    [`--${type.value}`]: true,
  };
});
const indentPx = computed(() => `${15 * props.level}px`);

const isEmpty = computed(() => {
  if (type.value === "array") return (props.value as Array<any>).length === 0;
  if (type.value === "object") return _.keys(props.value as any).length === 0;
  return _.isNil(props.value);
});
const leftBracket = computed(() => {
  if (type.value === "array") return "[";
  if (type.value === "object") return "{";
  return "";
});
const rightBracket = computed(() => {
  if (type.value === "array") return "]";
  if (type.value === "object") return "}";
  return "";
});
const childSummary = computed(() => {
  if (isEmpty.value) return "emtpy";
  if (type.value === "array") {
    const numItems = (props.value as any).length;
    if (!numItems) return "empty";
    return `${numItems} item${numItems > 1 ? "s" : ""}`;
  } else if (type.value === "object") {
    const itemKeys = _.keys(props.value);
    const numKeys = itemKeys.length;
    if (numKeys <= props.numPreviewProps) {
      return itemKeys.slice(0, props.numPreviewProps).join(", ");
    }
    return `${itemKeys.slice(0, props.numPreviewProps).join(", ")}, +${
      numKeys - props.numPreviewProps
    } more...`;
  }
  // should not reach here
  return "";
});
const stringValue = computed(() => {
  if (props.value === null) return "null";
  if (props.value === undefined) return "undefined";
  return props.value.toString();
});

onMounted(() => {
  // children listen to the rootnode for an open/close all event
  rootCtx.eventBus.on("toggleAllOpen", toggleOpen);
});

function toggleOpen(newIsOpen?: boolean) {
  if (canHaveChildren.value && !isEmpty.value) {
    if (_.isBoolean(newIsOpen)) isOpen.value = newIsOpen;
    else isOpen.value = !isOpen.value;
  }
}
</script>

<style lang="less">
.json-tree-explorer__row {
  .json-tree-explorer__row-inner {
    position: relative;
    display: flex;
    align-content: flex-start;
    padding: 3px 0;
    padding-left: 30px;
    cursor: pointer;
    &:hover {
      // highlight matching brackets
      > .json-tree-explorer__bracket,
      ~ .json-tree-explorer__bracket {
        opacity: 1;
        color: #4149e0;
      }
      .json-tree-explorer__extract-button {
        display: block;
      }
    }
  }
  .json-tree-explorer__collapse-toggle {
    margin-right: 10px;
    position: absolute;
    left: 0px;
    top: 0px;
    color: #555;
  }
  .json-tree-explorer__extract-button {
    position: absolute;
    right: 10px;
    display: none;
    box-shadow: rgba(0, 0, 0, 0.2) 0 1px 3px;
    z-index: 2;
  }

  .json-tree-explorer__prop-name {
    padding-right: 15px;
    &.--array-item {
      font-family: monospace;
      font-style: italic;
    }
  }
  .json-tree-explorer__value {
    padding-right: 10px;
    // white-space: pre-wrap;
    // word-wrap: break-word;
    // word-break: break-all;
    // white-space: normal;

    &.--number {
    }
    &.--string {
      color: #555;
    }
    &.--boolean {
    }
    &.--empty {
      opacity: 0.25;
    }
    &.--link {
      text-decoration: underline;
      color: #1e88e5;
      &:hover {
        color: darken(#1e88e5, 15%);
      }
    }
  }
  .json-tree-explorer__bracket {
    opacity: 0.3;
    &.--newline {
      margin-left: 30px;
    }
  }
  .json-tree-explorer__children {
    position: relative;
  }
  .json-tree-explorer__children_summary {
    opacity: 0.5;
    padding: 0 5px;
    &.--empty {
      opacity: 0.25;
    }
  }

  .json-tree-explorer__children-border-left {
    background: #dedede;
    width: 1px;
    position: absolute;
    left: 31px;
    top: 2px;
    bottom: 2px;
  }
}
</style>
