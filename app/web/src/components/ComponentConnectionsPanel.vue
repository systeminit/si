<template>
  <div class="flex flex-col min-h-full">
    <AttributesPanelCustomInputs />
    <div
      :class="
        clsx(
          'relative grow pb-md',
          `before:absolute before:w-6 before:left-0 before:top-0 before:bottom-0 before:content-['']`,
          themeClasses('before:bg-neutral-100', 'before:bg-neutral-900'),
        )
      "
    >
      <TreeForm :trees="trees" />
    </div>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { PropertyEditorPropKind } from "@/api/sdf/dal/property_editor";
import TreeForm from "./AttributesPanel/TreeForm.vue";
import { TreeFormData, TreeFormProp } from "./AttributesPanel/TreeFormItem.vue";
import AttributesPanelCustomInputs from "./AttributesPanel/AttributesPanelCustomInputs.vue";

// TODO - mock data, will need to plump through real data!
const lineageTree = {
  propDef: {
    id: "lineage",
    name: "lineage",
    icon: "socket",
    kind: PropertyEditorPropKind.Object,
    widgetKind: { kind: "header" },
    isHidden: false,
    isReadonly: false,
  } as TreeFormProp,
  children: [
    {
      propDef: {
        id: "parent",
        name: "parent",
        icon: "none",
        kind: PropertyEditorPropKind.String,
        widgetKind: { kind: "select" },
        isHidden: false,
        isReadonly: false,
      } as TreeFormProp,
      children: [],
      value: undefined,
      valueId: "parent",
      parentValueId: "parent",
      validation: null,
      propId: "parent",
    },
  ],
  value: undefined,
  valueId: "lineage",
  parentValueId: "lineage",
  validation: null,
  propId: "lineage",
} as TreeFormData;

const generateMockSockets = () => {
  const sockets = [];

  for (let i = 0; i < 5; i++) {
    sockets.push({
      propDef: {
        id: `socket${i}`,
        name: `example socket ${i}`,
        icon: "none",
        kind: PropertyEditorPropKind.String,
        widgetKind: { kind: "select" },
        isHidden: false,
        isReadonly: false,
      } as TreeFormProp,
      children: [],
      value: undefined,
      valueId: `socket${i}`,
      parentValueId: `socket${i}`,
      validation: null,
      propId: `socket${i}`,
    });
  }

  return sockets;
};

const socketsTree = {
  propDef: {
    id: "sockets",
    name: "sockets",
    icon: "socket",
    kind: PropertyEditorPropKind.Object,
    widgetKind: { kind: "header" },
    isHidden: false,
    isReadonly: false,
  } as TreeFormProp,
  children: [
    {
      propDef: {
        id: "outputs",
        name: "output sockets",
        icon: "output-socket",
        kind: PropertyEditorPropKind.Object,
        widgetKind: { kind: "header" },
        isHidden: false,
        isReadonly: false,
      } as TreeFormProp,
      children: generateMockSockets(),
      value: undefined,
      valueId: "outputs",
      parentValueId: "outputs",
      validation: null,
      propId: "outputs",
    },
    {
      propDef: {
        id: "inputs",
        name: "input sockets",
        icon: "input-socket",
        kind: PropertyEditorPropKind.Object,
        widgetKind: { kind: "header" },
        isHidden: false,
        isReadonly: false,
      } as TreeFormProp,
      children: generateMockSockets(),
      value: undefined,
      valueId: "inputs",
      parentValueId: "inputs",
      validation: null,
      propId: "inputs",
    },
  ],
  value: undefined,
  valueId: "sockets",
  parentValueId: "sockets",
  validation: null,
  propId: "sockets",
} as TreeFormData;

const trees = [lineageTree, socketsTree];
</script>
