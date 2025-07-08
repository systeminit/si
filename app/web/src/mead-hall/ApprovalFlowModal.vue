<template>
  <div>
    <!-- this modal is for the voting process -->
    <Modal ref="modalRef" title="Changes To Be Applied">
      <div class="max-h-[70vh] overflow-hidden flex flex-col">
        <div class="text-md mb-xs">
          Applying this change set may create, modify, or destroy real resources
          in the cloud.
        </div>
        <div class="text-sm mb-sm">
          These actions will be applied to the real world:
        </div>
        <div
          class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
        >
          <ActionsList
            slim
            kind="proposed"
            noInteraction
            :proposedActions="allActionViews"
          />
        </div>
        <div
          class="flex flex-row w-full items-center justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            icon="x"
            variant="ghost"
            tone="warning"
            @click="closeModalHandler"
          />
          <VButton
            icon="tools"
            :tone="workspaceHasOneUser ? 'success' : undefined"
            :label="
              workspaceHasOneUser ? 'Apply Change Set' : 'Request Approval'
            "
            class="grow"
            @click="applyButtonHandler"
          />
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VButton, Modal } from "@si/vue-lib/design-system";
import { computed, ref, watch } from "vue";
import { useToast } from "vue-toastification";
import { useQuery } from "@tanstack/vue-query";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ApprovalFlowCancelled from "@/components/toasts/ApprovalFlowCancelled.vue";
import { usePresenceStore } from "@/store/presence.store";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { ActionProposedView } from "@/store/actions.store";
import { BifrostActionViewList } from "@/workers/types/entity_kind_types";
import ActionsList from "./ActionsList.vue";
import { ActionProposedViewWithHydratedChildren } from "./ChangesPanelProposed.vue";

const presenceStore = usePresenceStore();
const changeSetsStore = useChangeSetsStore();
const authStore = useAuthStore();
const toast = useToast();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);
const changeSet = computed(() => changeSetsStore.selectedChangeSet);
const workspaceHasOneUser = computed(() => authStore.workspaceHasOneUser);

async function openModalHandler() {
  if (changeSet?.value?.name === "HEAD") return;

  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

function applyButtonHandler() {
  if (workspaceHasOneUser.value && authStore.user) {
    changeSetsStore.APPLY_CHANGE_SET(authStore.user.name);
  } else {
    changeSetsStore.REQUEST_CHANGE_SET_APPROVAL();

    // TODO(nick): we should remove this in favor of only the WsEvent fetching. It appears that
    // requesting the approval itself is insufficient for getting the latest approval status at
    // the time of writing and the reason appears to be that the change set is "open" by the
    // time the inset modal opens. Fortunately, this will work since we are the requester.
    if (changeSet.value) {
      changeSetsStore.FETCH_APPROVAL_STATUS(changeSet.value.id);
    }

    presenceStore.leftDrawerOpen = false; // close the left draw for the InsetModal
  }

  closeModalHandler();
}

const queryKey = makeKey("ActionViewList");
const actionViewList = useQuery<BifrostActionViewList | null>({
  queryKey,
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(makeArgs("ActionViewList")),
});
const allActionViews = computed(() => {
  if (!actionViewList.data.value) return [];
  if (actionViewList.data.value.actions.length < 1) return [];
  const proposed = actionViewList.data.value.actions;
  const proposedById = proposed.reduce(
    (obj, p) => {
      obj[p.id] = p;
      return obj;
    },
    {} as Record<string, ActionProposedView>,
  );
  return proposed.map((_p) => {
    const p = { ..._p } as ActionProposedViewWithHydratedChildren;
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    p.dependentOnActions = p.dependentOn.map((d) => proposedById[d]!);
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    p.myDependentActions = p.myDependencies.map((d) => proposedById[d]!);
    p.holdStatusInfluencedByActions = p.holdStatusInfluencedBy.map(
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      (d) => proposedById[d]!,
    );
    return p;
  });
});

watch(
  () => changeSetsStore.selectedChangeSet?.status,
  (newVal, oldVal) => {
    if (
      newVal === ChangeSetStatus.Open &&
      (oldVal === ChangeSetStatus.NeedsApproval ||
        oldVal === ChangeSetStatus.Approved ||
        oldVal === ChangeSetStatus.Rejected)
    ) {
      if (!changeSetsStore.headSelected) {
        toast({
          component: ApprovalFlowCancelled,
          props: {
            action: "applying",
          },
        });
      }
    }
  },
);
defineExpose({ open: openModalHandler });
</script>
