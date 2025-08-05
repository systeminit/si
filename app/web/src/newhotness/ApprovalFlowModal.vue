<template>
  <div>
    <Modal
      ref="modalRef"
      hideExitButton
      title="Changes To Be Applied"
      :size="
        changeSet && changeSet.status === ChangeSetStatus.NeedsApproval
          ? '4xl'
          : 'md'
      "
    >
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
        <InsetApprovalModal
          v-if="
            changeSet &&
            changeSet.status === ChangeSetStatus.NeedsApproval &&
            ctx.user
          "
          :changeSet="changeSet"
          :approvalData="approvalData"
          :workspaceUsers="workspaceUsers"
          :user="ctx.user"
        />
        <div
          v-else
          class="flex flex-row w-full items-center justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            tone="neutral"
            pill="Esc"
            @click="closeModalHandler"
          />
          <VButton
            :label="approvalsEnabled ? 'Request Approval' : 'Apply Change Set'"
            :class="
              clsx(
                'grow !text-sm !border !cursor-pointer !px-xs',
                themeClasses(
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                ),
              )
            "
            :loadingText="approvalsEnabled ? 'undefined' : 'Applying Changes'"
            :loading="approvalsEnabled ? false : applyChangeSet.loading.value"
            :disabled="approvalsEnabled ? false : disableApplyChangeSet"
            pill="Cmd + Enter"
            @click="debouncedApplyOrRequestApproval"
          />
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  PillCounter,
  Icon,
  VButton,
  Modal,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useRouter, useRoute } from "vue-router";
import { computed, onBeforeUnmount, inject, onMounted, ref } from "vue";
import { debounce } from "lodash-es";
import { useToast, POSITION } from "vue-toastification";
import { useQuery } from "@tanstack/vue-query";
import { ActionProposedView } from "@/store/actions.store";
import { ActionKind } from "@/api/sdf/dal/action";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { WorkspaceUser } from "@/store/auth.store";
import { ApprovalData } from "@/store/change_sets.store";
import { keyEmitter } from "./logic_composables/emitters";
import ActionCard from "./ActionCard.vue";
import InsetApprovalModal from "./InsetApprovalModal.vue";
import { reset } from "./logic_composables/navigation_stack";
import { useApplyChangeSet } from "./logic_composables/change_set";
import ToastApplyingChanges from "./nav/ToastApplyingChanges.vue";
import { useContext } from "./logic_composables/context";
import { useApi, routes } from "./api_composables";
import { Workspaces } from "./types";

const props = defineProps<{
  actions: ActionProposedView[];
}>();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

const ctx = useContext();

const changeSet = computed(() => ctx.changeSet.value);

// First, check if the workspace has the approvals features enabled at the Auth API level.
const workspaces = inject<Workspaces>("WORKSPACES");
const workspace = computed(() => {
  const maybeWorkspaces = workspaces?.workspaces?.value;
  if (!maybeWorkspaces) return undefined;
  return maybeWorkspaces[ctx.workspacePk.value];
});
const approvalsEnabledWithoutSoloUserCheck = computed(() => {
  if (!workspace.value) return false;
  return workspace.value.approvalsEnabled;
});

// Second, check if we are in a solo user workspace.
const usersApi = useApi(ctx);
const workspaceUsersQuery = useQuery<Record<string, WorkspaceUser>>({
  enabled: () => approvalsEnabledWithoutSoloUserCheck.value,
  queryKey: ["workspacelistusers"],
  staleTime: 5000,
  queryFn: async () => {
    const call = usersApi.endpoint<{ users: WorkspaceUser[] }>(
      routes.WorkspaceListUsers,
    );
    const response = await call.get();
    if (usersApi.ok(response)) {
      return _.keyBy(response.data.users, "id");
    }
    return {} as Record<string, WorkspaceUser>;
  },
});
const workspaceUsers = computed(() => workspaceUsersQuery.data.value ?? {});
const isSoloUserWorkspace = computed(
  () => Object.keys(workspaceUsers.value).length === 1,
);

// Third, combine the two checks to determine if we should allow users to request approval.
const approvalsEnabled = computed(
  () =>
    approvalsEnabledWithoutSoloUserCheck.value && !isSoloUserWorkspace.value,
);

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
      debouncedApplyOrRequestApproval();
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

const debouncedApplyOrRequestApproval = debounce(applyOrRequestApproval, 500);
onBeforeUnmount(() => {
  debouncedApplyOrRequestApproval.cancel();
});

const toast = useToast();

const requestApprovalApi = useApi(ctx);

async function applyOrRequestApproval() {
  if (approvalsEnabled.value) {
    const requestApprovalCall = requestApprovalApi.endpoint(
      routes.ChangeSetRequestApproval,
    );
    requestApprovalCall.post({});
  } else {
    const result = await applyChangeSet.performApply();
    if (result.success) {
      closeModalHandler();
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
    }
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

const approvalDataApi = useApi(ctx);
const approvalDataQuery = useQuery<ApprovalData | undefined>({
  enabled: () => approvalsEnabled.value,
  queryKey: ["approvalstatus", ctx.changeSetId.value],
  queryFn: async () => {
    const call = approvalDataApi.endpoint<ApprovalData>(
      routes.ChangeSetApprovalStatus,
    );
    const response = await call.get();
    if (approvalDataApi.ok(response)) {
      return response.data;
    }
    return undefined;
  },
});
const approvalData = computed(() => approvalDataQuery.data.value);

defineExpose({ open: openModalHandler });
</script>
