<template>
  <div>
    <Modal ref="modalRef" hideExitButton title="Changes To Be Applied">
      <div class="max-h-[70vh] overflow-hidden flex flex-col">
        <div class="text-sm mb-xs pb-sm">
          Applying this change set may create, modify, or destroy real resources
          in the cloud. These actions will be applied to the real world:
        </div>
        <div
          class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
        >
          <div class="flex flex-row py-xs">
            <span class="ml-xs text-md">{{ actionsTitle }}</span>

            <!-- NOTE(nick): these are right-aligned pill counters for each action kind. -->
            <div class="ml-auto mr-xs flex flex-row">
              <PillCounter hideIfZero class="ml-2xs" :count="counts.create">
                <Icon name="plus" tone="success" size="xs" />
              </PillCounter>
              <PillCounter hideIfZero class="ml-2xs" :count="counts.destroy">
                <Icon name="x" tone="destructive" size="xs" />
              </PillCounter>
              <PillCounter hideIfZero class="ml-2xs" :count="counts.refresh">
                <Icon name="refresh" tone="action" size="xs" />
              </PillCounter>
              <PillCounter hideIfZero class="ml-2xs" :count="counts.other">
                <Icon name="play" tone="warning" size="xs" />
              </PillCounter>
            </div>
          </div>
          <ul class="actions list">
            <!-- NOTE(nick): we are re-using the action cards, but are disallowing interaction. -->
            <ActionCard
              v-for="action in props.actions"
              :key="action.id"
              :action="action"
              :selected="false"
              noInteraction
            />
          </ul>
        </div>
        <div
          class="flex flex-row w-full items-center justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            tone="neutral"
            pill="Esc"
            @click="closeModalHandler"
          />
          <!--
          TODO(nick): restore the dynamic label when approvals are re-introduced.
          ```
          :label="
            workspaceHasOneUser || !workspacesStore.workspaceApprovalsEnabled
            ? 'Apply Change Set'
            : 'Request Approval'
          "
          ```
          -->
          <VButton
            tone="action"
            label="Apply Change Set"
            class="grow"
            loadingText="Applying Changes"
            :loading="applyChangeSet.loading.value"
            :disabled="disableApplyChangeSet"
            pill="Cmd + Enter"
            @click="debouncedApply"
          />
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { PillCounter, Icon, VButton, Modal } from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import { computed, inject, onBeforeUnmount, onMounted, ref } from "vue";
import { debounce } from "lodash-es";
import { useToast, POSITION } from "vue-toastification";
import { ActionProposedView } from "@/store/actions.store";
import { ActionKind } from "@/api/sdf/dal/action";
import { keyEmitter } from "./logic_composables/emitters";
import ActionCard from "./ActionCard.vue";
import { assertIsDefined, Context } from "./types";
import { reset } from "./logic_composables/navigation_stack";
import { useApplyChangeSet } from "./logic_composables/change_set";
import ToastApplyingChanges from "./nav/ToastApplyingChanges.vue";

const props = defineProps<{
  actions: ActionProposedView[];
}>();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const router = useRouter();
const route = useRoute();

const actionsTitle = computed(() =>
  props.actions.length === 1
    ? `${props.actions.length} Action`
    : `${props.actions.length} Actions`,
);

const counts = computed(() => {
  const results: Record<string, number> = {
    create: 0,
    destroy: 0,
    refresh: 0,
    other: 0, // NOTE(nick): "manual" and "other" are grouped together
  };
  for (const action of props.actions) {
    if (action.kind === ActionKind.Create) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      results.create! += 1;
    } else if (action.kind === ActionKind.Destroy) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      results.destroy! += 1;
    } else if (action.kind === ActionKind.Refresh) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      results.refresh! += 1;
    } else {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      results.other! += 1;
    }
  }
  return results;
});

const clearKeyEmitters = () => {
  keyEmitter.off("Enter");
};
onMounted(() => {
  clearKeyEmitters();

  keyEmitter.on("Enter", (e) => {
    if (e.metaKey || e.ctrlKey) {
      debouncedApply();
    }
  });
});
onBeforeUnmount(() => {
  clearKeyEmitters();
});

async function openModalHandler() {
  if (ctx?.onHead.value) return;

  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

const { applyChangeSet, disableApplyChangeSet } = useApplyChangeSet();

const debouncedApply = debounce(apply, 500);
onBeforeUnmount(() => {
  debouncedApply.cancel();
});

const toast = useToast();

async function apply() {
  // TODO(nick): restore approvals in the new UI.
  // if (!workspacesStore.workspaceApprovalsEnabled && authStore.user) {
  //   changeSetsStore.APPLY_CHANGE_SET_NEW_HOTNESS(authStore.user.name);
  // } else {
  //   if (workspaceHasOneUser.value && authStore.user) {
  //     changeSetsStore.APPLY_CHANGE_SET_NEW_HOTNESS(authStore.user.name);
  //   } else {
  //     changeSetsStore.REQUEST_CHANGE_SET_APPROVAL();
  //
  //     // TODO(nick): we should remove this in favor of only the WsEvent fetching. It appears that
  //     // requesting the approval itself is insufficient for getting the latest approval status at
  //     // the time of writing and the reason appears to be that the change set is "open" by the
  //     // time the inset modal opens. Fortunately, this will work since we are the requester.
  //     if (changeSet.value) {
  //       changeSetsStore.FETCH_APPROVAL_STATUS(changeSet.value.id);
  //     }
  //
  //     presenceStore.leftDrawerOpen = false; // close the left draw for the InsetModal
  //   }
  // }
  if (!ctx) return;

  const result = await applyChangeSet.performApply();
  if (result.success) {
    toast(
      {
        component: ToastApplyingChanges,
      },
      {
        position: POSITION.BOTTOM_CENTER,
        timeout: 5000,
      },
    );
    const name = route.name;
    router.push({
      name,
      params: {
        ...route.params,
        changeSetId: ctx.headChangeSetId.value,
      },
      query: route.query,
    });
    reset();
    closeModalHandler();
  }
}

// NOTE(nick): this should only be relevant when approval requirements come back.
// watch(
//   () => changeSetsStore.selectedChangeSet?.status,
//   (newVal, oldVal) => {
//     if (
//       newVal === ChangeSetStatus.Open &&
//       (oldVal === ChangeSetStatus.NeedsApproval ||
//         oldVal === ChangeSetStatus.Approved ||
//         oldVal === ChangeSetStatus.Rejected)
//     ) {
//       if (!changeSetsStore.headSelected) {
//         toast({
//           component: ApprovalFlowCancelled,
//           props: {
//             action: "applying",
//           },
//         });
//       }
//     }
//   },
// );

defineExpose({ open: openModalHandler });
</script>
