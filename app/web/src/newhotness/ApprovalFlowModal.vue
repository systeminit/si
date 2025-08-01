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
        <ApprovalStatusBox
          v-if="changeSet && approvalStatus && mode && user"
          :approvalData="approvalStatus"
          :changeSet="changeSet"
          :mode="mode"
          :user="user"
          :workspaceUsers="workspaceUsers"
          @approve="performApproveOrReject('Approved')"
          @reject="performApproveOrReject('Rejected')"
          @withdraw="performWithdraw"
        />
        <div
          class="flex flex-row w-full items-center justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            tone="neutral"
            pill="Esc"
            @click="closeModalHandler"
          />
          <VButton
            :label="buttonLabel"
            :class="
              clsx(
                'grow !text-sm !border !cursor-pointer !px-xs',
                themeClasses(
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                ),
              )
            "
            loadingText="Applying Changes"
            :loading="loading"
            :disabled="disableApply"
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
import {
  PillCounter,
  Icon,
  VButton,
  Modal,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useRouter, useRoute } from "vue-router";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { debounce } from "lodash-es";
import { useStatus } from "./logic_composables/status";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useToast, POSITION } from "vue-toastification";
import { useQuery } from "@tanstack/vue-query";
import { ActionProposedView } from "@/store/actions.store";
import { ActionKind } from "@/api/sdf/dal/action";
import { keyEmitter } from "./logic_composables/emitters";
import ActionCard from "./ActionCard.vue";
import { reset } from "./logic_composables/navigation_stack";
import ToastApplyingChanges from "./nav/ToastApplyingChanges.vue";
import ApprovalStatusBox from "./ApprovalStatusBox.vue";
import { useContext } from "./logic_composables/context";
import {
  useApplyChangeSet,
  useApprovalStatus,
  useApproveOrReject,
} from "./logic_composables/change_set";
import { routes, useApi } from "./api_composables";

const props = defineProps<{
  actions: ActionProposedView[];
}>();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

const ctx = useContext();
const approvalsEnabled = computed(
  () => ctx.approvalsEnabled.value && !ctx.workspaceHasOneUser.value,
);
const changeSet = computed(() => ctx.changeSet.value);
const user = computed(() => ctx.user);
const workspaceUsers = computed(() => {
  console.log(
    "NICK WORKSPACE USERS",
    JSON.stringify(ctx.workspaceUsers.value, null, 2),
  );
  return {};
});

const approvalStatus = useApprovalStatus(computed(() => ctx));

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

const { loading, performApply } = useApplyChangeSet(ctx);

const status = useStatus();
const disableApply = computed(
  () =>
    (approvalsEnabled.value &&
      ctx.changeSet.value?.status === ChangeSetStatus.NeedsApproval &&
      mode.value !== "requested") ||
    (ctx.changeSet.value?.status !== ChangeSetStatus.Open &&
      ctx.changeSet.value?.status !== ChangeSetStatus.NeedsApproval) ||
    ctx.onHead.value ||
    status.value === "syncing",
);

const debouncedApply = debounce(apply, 500);
onBeforeUnmount(() => {
  debouncedApply.cancel();
});

const toast = useToast();

const changeSetStatus = computed(() => ctx.changeSet.value?.status);
const mode = computed(() => {
  if (!approvalStatus.value || !changeSetStatus.value) return undefined;

  const satisfied = !approvalStatus.value.requirements.some(
    (r) => r.isSatisfied === false,
  );

  if (satisfied) return "approved";
  switch (changeSetStatus.value) {
    case ChangeSetStatus.NeedsApproval:
      return "requested";
    case ChangeSetStatus.Approved:
      return "approved";
    case ChangeSetStatus.Rejected:
      return "rejected";
    default:
      return "error";
  }
});

const buttonLabel = computed(() => {
  if (!approvalsEnabled.value || mode.value === "approved")
    return "Apply Change Set";
  return "Request Approval";
});

async function apply() {
  if (approvalsEnabled.value && mode.value !== "approved") {
    const success = await requestApproval();
    if (success) closeModalHandler();
  } else {
    const result = await performApply();
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

const requestApprovalApi = useApi();
const requestApproval = async () => {
  const requestApprovalCall = requestApprovalApi.endpoint(
    routes.ChangeSetRequestApproval,
  );
  const response = await requestApprovalCall.post({});
  return requestApprovalApi.ok(response.req);
};

const rejected = computed(() => mode.value === "rejected");
const performApproveOrReject = useApproveOrReject(ctx);

const reopenApi = useApi(ctx);
const cancelApi = useApi(ctx);

const performWithdraw = async () => {
  if (rejected.value) {
    const reopenCall = reopenApi.endpoint(routes.ChangeSetReopen);
    reopenCall.post({});
  } else {
    const cancelCall = cancelApi.endpoint(
      routes.ChangeSetCancelApprovalRequest,
    );
    cancelCall.post({});
  }
};

defineExpose({ open: openModalHandler });
</script>
