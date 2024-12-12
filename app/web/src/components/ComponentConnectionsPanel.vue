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
      <TreeForm
        :trees="trees"
        @setValue="setValueHandler"
        @unsetValue="resetHandler"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { computed } from "vue";
import * as _ from "lodash-es";
import { PropertyEditorPropKind } from "@/api/sdf/dal/property_editor";
import {
  generateEdgeId,
  SocketWithParent,
  SocketWithParentAndEdge,
  useComponentsStore,
} from "@/store/components.store";
import { LabelEntry, LabelList } from "@/api/sdf/dal/label_list";
import { useViewsStore } from "@/store/views.store";
import {
  DiagramNodeDef,
  DiagramSocketDef,
  DiagramViewData,
} from "@/components/ModelingDiagram/diagram_types";
import { ComponentType } from "@/api/sdf/dal/schema";
import TreeForm from "./AttributesPanel/TreeForm.vue";
import { TreeFormData, TreeFormProp } from "./AttributesPanel/TreeFormItem.vue";
import AttributesPanelCustomInputs from "./AttributesPanel/AttributesPanelCustomInputs.vue";

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();

// PARENTS
const parentOptionsList = computed(() => {
  const selectedComponentId = viewsStore.selectedComponentId;
  const groups = Object.values(componentsStore.groupsById);
  const list = [] as LabelList<string>;

  groups.forEach((group) => {
    if (group.def.id !== selectedComponentId) {
      list.push({
        label: `${group.def.displayName} (${group.def.schemaName})`,
        value: group.def.id,
      } as LabelEntry<string>);
    }
  });

  return list;
});

const currentParentNamePropValue = computed(() => {
  if (!currentParent.value) {
    return null;
  }

  return {
    id: currentParent.value?.id,
    propId: currentParent.value?.id,
    value: currentParent.value
      ? `${currentParent.value.displayName} (${currentParent.value.schemaName})`
      : null,
    canBeSetBySocket: false,
    isFromExternalSource: false,
    isControlledByDynamicFunc: false,
    isControlledByAncestor: false,
    overridden: true,
    ancestorManual: false,
  };
});

const lineageTree = computed(
  () =>
    ({
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
            id: viewsStore.selectedComponentId,
            name: "parent",
            icon: "none",
            kind: PropertyEditorPropKind.String,
            widgetKind: { kind: "select", options: parentOptionsList.value },
            isHidden: false,
            isReadonly: false,
          } as TreeFormProp,
          children: [],
          value: currentParentNamePropValue.value,
          valueId: currentParent.value?.id,
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
    } as TreeFormData),
);

const currentParent = computed(() => {
  const selectedComponent = viewsStore.selectedComponent;

  if (!selectedComponent || selectedComponent instanceof DiagramViewData) {
    return;
  }

  const parentId = selectedComponent.def.parentId;

  if (!parentId) {
    return;
  }

  return componentsStore.groupsById[parentId]?.def;
});

// SOCKETS
const treeFormItemFromSocket = (
  socket: DiagramSocketDef,
  component: DiagramNodeDef,
  existingPeers: SocketWithParentAndEdge[],
  possiblePeers: SocketWithParent[],
) => {
  const combinedId = `${component.id}-${socket.id}`;
  const headerId = `${combinedId}-header`;

  return {
    propDef: {
      id: headerId,
      name: socket.label,
      icon: socket.nodeSide === "left" ? "input-socket" : "output-socket",
      kind: PropertyEditorPropKind.Object,
      widgetKind: { kind: "header" },
      isHidden: false,
      isReadonly: false,
    } as TreeFormProp,
    children: [
      {
        propDef: {
          id: combinedId,
          name: socket.label,
          icon: "none",
          kind: PropertyEditorPropKind.String,
          widgetKind: {
            kind: "socketConnection",
            options: possiblePeers.map((peerSocket) => ({
              label: peerSocket.componentName,
              label2: peerSocket.label,
              value: `${peerSocket.componentId}-${peerSocket.id}`,
              componentId: peerSocket.componentId,
            })),
            isSingleArity:
              socket.nodeSide === "left" && socket.maxConnections === 1,
            isUpFrameInput:
              component.componentType === ComponentType.ConfigurationFrameUp &&
              socket.nodeSide === "left",
          },
          isHidden: false,
          isReadonly: false,
        } as TreeFormProp,
        children: [],
        value: {
          id: combinedId,
          propId: combinedId,
          value: existingPeers.map((peerSocket) => ({
            label: peerSocket.componentName,
            label2: peerSocket.label,
            value: `${peerSocket.componentId}-${peerSocket.id}`,
            isInferred: peerSocket.edge.isInferred,
          })),
          canBeSetBySocket: false,
          isFromExternalSource: false,
          isControlledByDynamicFunc: false,
          isControlledByAncestor: false,
          overridden: false,
          ancestorManual: false,
        },
        valueId: combinedId,
        parentValueId:
          socket.nodeSide === "left" ? "inputSockets" : "outputSockets",

        validation: null,
        propId: combinedId,
      },
    ],
    value: undefined,
    valueId: headerId,
    parentValueId:
      socket.nodeSide === "left" ? "inputSockets" : "outputSockets",
    validation: null,
    propId: headerId,
  };
};

const sockets = computed(() => {
  const selectedComponent = viewsStore.selectedComponent;

  if (
    !selectedComponent ||
    selectedComponent instanceof DiagramViewData ||
    !selectedComponent.def.sockets
  ) {
    return { input: [], output: [] };
  }

  const peersFunction = componentsStore.possibleAndExistingPeerSocketsFn;
  const sockets =
    selectedComponent.def.sockets
      .filter((s) => !s.isManagement)
      .map((s) => {
        const { existingPeers, possiblePeers } = peersFunction(
          s,
          selectedComponent.def.id,
        );

        return treeFormItemFromSocket(
          s,
          selectedComponent.def,
          existingPeers,
          possiblePeers,
        );
      }) ?? [];

  const [input, output] = _.partition(
    sockets,
    (s) => s.parentValueId === "inputSockets",
  );

  return { input, output };
});

const generateSocketsTree = (
  tree: TreeFormData[],
  direction: "input" | "output",
) => {
  const id = `${direction}Sockets`;
  const directionString =
    direction.charAt(0).toUpperCase() + direction.slice(1);
  const name = `${tree.length} ${directionString} Socket${
    tree.length === 1 ? "" : "s"
  }`;

  return {
    propDef: {
      id,
      name,
      icon: "socket",
      kind: PropertyEditorPropKind.Object,
      widgetKind: { kind: "header" },
      isHidden: false,
      isReadonly: false,
    } as TreeFormProp,
    children: tree,
    value: undefined,
    valueId: id,
    parentValueId: "connections",
    validation: null,
    propId: id,
  } as TreeFormData;
};

const inputSocketsTree = computed(() =>
  generateSocketsTree(sockets.value.input, "input"),
);
const outputSocketsTree = computed(() =>
  generateSocketsTree(sockets.value.output, "output"),
);

const trees = computed(() => [
  lineageTree.value,
  inputSocketsTree.value,
  outputSocketsTree.value,
]);

const resetHandler = (item: TreeFormData, value?: string) => {
  if (item.propId === "parent") {
    viewsStore.SET_PARENT([item.propDef.id], null);
  }

  if (!value) return;
  const [thisComponentId, thisSocketId] = item.propId.split("-");
  const [otherComponentId, otherSocketId] = value.split("-");
  if (
    !thisComponentId ||
    !thisSocketId ||
    !otherComponentId ||
    !otherSocketId
  ) {
    return;
  }

  const [from, to] =
    item.parentValueId === "inputSockets"
      ? [
          {
            componentId: otherComponentId,
            socketId: otherSocketId,
          },
          {
            componentId: thisComponentId,
            socketId: thisSocketId,
          },
        ]
      : [
          {
            componentId: thisComponentId,
            socketId: thisSocketId,
          },
          {
            componentId: otherComponentId,
            socketId: otherSocketId,
          },
        ];

  const edgeId = generateEdgeId(
    from.componentId,
    to.componentId,
    from.socketId,
    to.socketId,
  );

  useComponentsStore().DELETE_EDGE(
    edgeId,
    to.socketId,
    from.socketId,
    to.componentId,
    from.componentId,
  );
};

const setValueHandler = (item: TreeFormData, value: string) => {
  if (item.propId === "parent") {
    viewsStore.SET_PARENT([item.propDef.id], value);
    return;
  }

  const [thisComponentId, thisSocketId] = item.propId.split("-");
  const [otherComponentId, otherSocketId] = value.split("-");
  if (
    !thisComponentId ||
    !thisSocketId ||
    !otherComponentId ||
    !otherSocketId
  ) {
    return;
  }

  const [from, to] =
    item.parentValueId === "inputSockets"
      ? [
          {
            componentId: otherComponentId,
            socketId: otherSocketId,
          },
          {
            componentId: thisComponentId,
            socketId: thisSocketId,
          },
        ]
      : [
          {
            componentId: thisComponentId,
            socketId: thisSocketId,
          },
          {
            componentId: otherComponentId,
            socketId: otherSocketId,
          },
        ];

  componentsStore.CREATE_COMPONENT_CONNECTION(from, to);
};
</script>
