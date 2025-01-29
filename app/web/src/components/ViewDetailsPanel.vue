<template>
  <ScrollArea>
    <template #top>
      <ViewCard
        :view="selectedView"
        :displayAsComponentCard="selectedViewComponent"
      />
      <div
        :class="
          clsx(
            'flex flex-row p-xs',
            themeClasses('bg-neutral-100', 'bg-neutral-900'),
          )
        "
      >
        <div class="font-bold flex-1">Approval Requirements</div>
        <PillCounter
          v-if="listReq.isSuccess && listReq.completed"
          class="flex-none font-bold"
          :count="requirementsCount"
          hideIfZero
          :paddingX="requirementsCount < 10 ? 'xs' : '2xs'"
        />
      </div>
    </template>

    <template v-if="listReq.isSuccess && listReq.completed">
      <TreeForm v-if="requirementsCount > 0" :trees="requirementTrees" />
      <div v-else class="flex flex-col gap-xs items-center">
        <EmptyStateCard
          iconName="customize"
          primaryText="No Requirements For This View"
          secondaryText="To add an approval requirement for this view, select the first user to be an approver from the dropdown below."
        />
        <UserSelectMenu class="w-full px-sm" @select="createRequirement" />
      </div>
    </template>
    <LoadingMessage v-else message="Loading..." :requestStatus="listReq" />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  LoadingMessage,
  PillCounter,
  ScrollArea,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed, PropType, watch } from "vue";
import clsx from "clsx";
import {
  ViewApprovalRequirementDefinition,
  ViewDescription,
} from "@/api/sdf/dal/views";
import { PropertyEditorPropKind } from "@/api/sdf/dal/property_editor";
import { useViewsStore } from "@/store/views.store";
import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ViewCard from "./ViewCard.vue";
import { DiagramViewData } from "./ModelingDiagram/diagram_types";
import TreeForm from "./AttributesPanel/TreeForm.vue";
import { TreeFormData, TreeFormProp } from "./AttributesPanel/TreeFormItem.vue";
import EmptyStateCard from "./EmptyStateCard.vue";
import UserSelectMenu from "./UserSelectMenu.vue";

const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();
const viewsStore = useViewsStore();

const listReq = viewsStore.getRequestStatus("LIST_VIEW_APPROVAL_REQUIREMENTS");

const props = defineProps({
  selectedView: { type: Object as PropType<ViewDescription>, required: true },
  selectedViewComponent: {
    type: Object as PropType<DiagramViewData>,
    required: true,
  },
});

const rawRequirements = computed(
  () => viewsStore.requirementDefintionsByViewId[props.selectedView.id],
);

const requirementsCount = computed(() => {
  if (rawRequirements.value) {
    return rawRequirements.value.length;
  } else return 0;
});

const generateUserTree = (
  requirementDef: ViewApprovalRequirementDefinition,
) => {
  const n = requirementDef.approverIndividuals.length;
  const users = [];

  for (let i = 0; i < n; i++) {
    let isReadonly = false;
    if (n === 1) isReadonly = true;

    const id = requirementDef.approverIndividuals[i];
    if (!id) break;
    const user = workspaceUsersById.value[id];
    if (!user) break;
    const name = `${user.name} - ${user.email}`;
    users.push({
      propDef: {
        id,
        name,
        icon: "user-circle",
        kind: PropertyEditorPropKind.String,
        widgetKind: { kind: "users" },
        isHidden: false,
        isReadonly,
      } as TreeFormProp,
      children: [],
      value: undefined,
      valueId: id,
      parentValueId: requirementDef.id,
      validation: null,
      propId: id,
    } as TreeFormData);
  }

  return users;
};

const generateOneRequirementTree = (
  requirementDef: ViewApprovalRequirementDefinition,
  n: number,
  showNumber = true,
) =>
  ({
    propDef: {
      id: requirementDef.id,
      name: showNumber ? `requirement ${n}` : "requirement",
      icon: "bullet-list",
      kind: PropertyEditorPropKind.Object,
      widgetKind: { kind: "requirement" },
      isHidden: false,
      isReadonly: false,
    } as TreeFormProp,
    children: [
      {
        propDef: {
          id: `approvers${n}`,
          name: "approvers",
          icon: "user-circle",
          kind: PropertyEditorPropKind.Array,
          widgetKind: { kind: "users" },
          isHidden: false,
          isReadonly: false,
        } as TreeFormProp,
        children: generateUserTree(requirementDef),
        value: undefined,
        valueId: `approvers${n}`,
        parentValueId: requirementDef.id,
        validation: null,
        propId: `approvers${n}`,
      } as TreeFormData,
    ],
    value: undefined,
    valueId: requirementDef.id,
    parentValueId: "root",
    validation: null,
    propId: requirementDef.id,
  } as TreeFormData);

const requirementTrees = computed(() => {
  const trees = [] as TreeFormData[];
  const showNumber = requirementsCount.value > 1;

  if (!rawRequirements.value) return trees;

  for (let i = 0; i < requirementsCount.value; i++) {
    const requirementDef = rawRequirements.value[i];
    if (requirementDef) {
      trees.push(generateOneRequirementTree(requirementDef, i + 1, showNumber));
    } else break;
  }

  return trees;
});

const refreshData = () => {
  viewsStore.LIST_VIEW_APPROVAL_REQUIREMENTS(props.selectedView.id);
  if (changeSetsStore.selectedWorkspacePk) {
    authStore.LIST_WORKSPACE_USERS(changeSetsStore.selectedWorkspacePk);
  }
};

watch(
  () => props.selectedView.id,
  () => {
    refreshData();
  },
  { immediate: true },
);

const workspaceUsersById = computed(() => authStore.workspaceUsers);

const createRequirement = async (firstApproverUserId: string) => {
  await viewsStore.CREATE_VIEW_APPROVAL_REQUIREMENT(
    props.selectedView.id,
    firstApproverUserId,
  );
};
</script>
