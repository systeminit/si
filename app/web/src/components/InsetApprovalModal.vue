<template>
  <div
    v-if="mode !== 'error'"
    :class="
      clsx(
        'lg:w-1/2 flex flex-col gap-sm p-sm shadow-2xl max-h-full overflow-hidden',
        themeClasses('bg-shade-0 border', 'bg-neutral-900'),
      )
    "
  >
    <div class="flex flex-row flex-none gap-md mb-sm items-center">
      <div class="flex flex-col gap-2xs">
        <TruncateWithTooltip class="font-bold italic pb-2xs">
          {{ changeSetName }}
        </TruncateWithTooltip>
        <div class="font-bold">{{ modalData.title }}</div>
        <div v-if="modalData.date" class="text-sm italic">
          <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
        </div>
      </div>

      <ErrorMessage
        :tone="modalData.messageTone"
        :icon="modalData.messageIcon"
        variant="block"
        class="rounded grow"
      >
        <template v-if="mode === 'requested'">
          There are approvals that must be met before the change set can be
          applied.
        </template>
        <template
          v-else-if="
            !featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
            (mode === 'approved' || mode === 'rejected')
          "
        >
          {{ requesterIsYou ? "Your" : "The" }} request to
          <span class="font-bold">Apply</span> change set
          <span class="font-bold">{{ changeSetName }}</span> was {{ mode }} by
          <span class="font-bold">{{ approverEmail + " " }}</span>

          <!-- {{ modalData.date.getTime() === new Date().getTime() ? "" : "on" }} -->
          <span class="font-bold">
            <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
          </span>
          <div
            v-if="!requesterIsYou && !userIsApprover && mode === 'approved'"
            class="pt-xs"
          >
            <span class="font-bold">{{ requesterEmail }}</span> requested this
            <span class="font-bold">Apply</span> and can merge this change set.
            You can switch to a different change set using the dropdown at the
            top of the screen.
          </div>
        </template>
        <template
          v-else-if="
            featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
            mode === 'approved'
          "
        >
          <p>
            {{ requesterIsYou ? "Your" : "The" }} request to
            <span class="font-bold">Apply</span> change set
            <span class="font-bold">{{ changeSetName }}</span> has been
            approved.
          </p>
        </template>
        <template v-else>
          ERROR - this message should not ever show. Something has gone wrong!
        </template>
      </ErrorMessage>
    </div>
    <div
      :class="
        clsx(
          'flex flex-row flex-1 overflow-hidden',
          featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL
            ? 'place-content-evenly'
            : 'justify-center',
        )
      "
    >
      <template v-if="featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL">
        <div class="flex flex-col text-sm gap-sm overflow-y-auto">
          <div
            v-for="group in requirementGroups"
            :key="group.key"
            class="border-neutral-100 dark:border-neutral-700 border"
          >
            <p class="bg-neutral-100 dark:bg-neutral-700 p-xs">
              {{ group.requiredCount }} of the following users for
              {{ group.label }}
            </p>
            <ul class="p-xs">
              <li
                v-for="vote in group.votes"
                :key="vote.user.id"
                class="flex flex-row gap-xs place-content-between"
              >
                <span>{{ vote.user.name }} ({{ vote.user.email }})</span>
                <span class="flex flex-row">
                  <span class="italic pr-xs">
                    <template v-if="!vote.status">Waiting...</template>
                    <template v-else>{{ vote.status }}...</template>
                  </span>
                  <Icon
                    size="md"
                    name="thumbs-up"
                    tone="success"
                    :class="
                      clsx(vote.status !== 'Approved' ? 'opacity-20' : '')
                    "
                  />
                  <Icon
                    size="md"
                    name="thumbs-down"
                    tone="error"
                    :class="
                      clsx(vote.status !== 'Rejected' ? 'opacity-20' : '')
                    "
                  />
                </span>
              </li>
            </ul>
          </div>
        </div>
      </template>
      <div class="flex flex-col gap-xs overflow-hidden">
        <div
          v-if="!featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL"
          class="text-sm"
        >
          These actions will be applied to the real world:
        </div>
        <div
          class="flex-grow overflow-y-auto border border-neutral-100 dark:border-neutral-700 min-w-[250px]"
        >
          <ActionsList slim kind="proposed" noInteraction />
        </div>
      </div>
    </div>
    <div class="flex flex-row flex-none gap-sm justify-center mt-sm">
      <VButton
        label="Withdraw Request"
        tone="warning"
        variant="ghost"
        icon="x"
        @click="withdraw"
      />
      <template
        v-if="
          (featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
            userIsApprover) ||
          (!featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
            changeSetsStore.currentUserIsDefaultApprover)
        "
      >
        <VButton
          :disabled="
            (!featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
              mode !== 'rejected') ||
            iRejected
          "
          label="Reject Request"
          tone="destructive"
          icon="thumbs-down"
          @click="rejectHandler"
        />
        <VButton
          :disabled="mode !== 'requested' || iApproved"
          label="Approve Request"
          tone="success"
          icon="thumbs-up"
          @click="approve"
        />
      </template>
      <VButton
        :disabled="mode !== 'approved'"
        tone="success"
        :loading="mode === 'approved' ? applyingChangeSet : false"
        loadingText="Applying..."
        @click="apply"
      >
        <span class="dark:text-neutral-800">Apply Change Set</span>
        <template #icon>
          <Icon name="tools" size="sm" class="dark:text-neutral-800" />
        </template>
      </VButton>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  VButton,
  Timestamp,
  Tones,
  ErrorMessage,
  Icon,
  IconNames,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import {
  ApprovalData,
  ApprovalStatus,
  approverForChangeSet,
  useChangeSetsStore,
} from "@/store/change_sets.store";
import { useAuthStore, WorkspaceUser } from "@/store/auth.store";
import { ChangeSetStatus, ChangeSet } from "@/api/sdf/dal/change_set";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useViewsStore } from "@/store/views.store";
import { useAssetStore } from "@/store/asset.store";
import ActionsList from "./Actions/ActionsList.vue";

export type InsetApprovalModalMode =
  | "requested"
  | "approved"
  | "rejected"
  | "error";

const assetStore = useAssetStore();
const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();
const viewStore = useViewsStore();

const applyingChangeSet = ref(false);
const changeSetName = computed(() => changeSetsStore.selectedChangeSet?.name);

const props = defineProps<{
  approvalData: ApprovalData | undefined;
  changeSet: ChangeSet;
}>();

interface Requirement {
  key: string;
  label: string;
  votes: Vote[];
  satisfied: boolean;
  requiredCount: number;
}
interface Vote {
  user: WorkspaceUser;
  status?: ApprovalStatus;
}

const requirementGroups = computed(() => {
  const groups: Requirement[] = [];
  props.approvalData?.requirements.forEach((r) => {
    const userIds = Object.values(r.approverGroups)
      .flat()
      .concat(r.approverIndividuals);
    const votes: Vote[] = [];
    userIds.forEach((id) => {
      const user = authStore.workspaceUsers[id];
      if (!user) return;
      const submitted = props.approvalData?.latestApprovals.find(
        (a) =>
          a.isValid &&
          a.userId === id &&
          r.applicableApprovalIds.includes(a.id),
      );
      const vote: Vote = { user };
      if (submitted) vote.status = submitted.status;
      votes.push(vote);
    });

    let label = r.entityKind;
    if (r.entityKind === "ApprovalRequirementDefinition") {
      label = "Approval Requirement change";
    } else if (r.entityKind === "Schema") {
      const variantForSchema = assetStore.schemaVariants.find(
        (thing) => thing.schemaId === r.entityId,
      );
      label = variantForSchema?.schemaName
        ? `Asset named ${variantForSchema?.schemaName}`
        : "an Asset";
    } else if (r.entityKind === "SchemaVariant") {
      let name = assetStore.variantFromListById[r.entityId]?.displayName;
      if (!name) {
        name = assetStore.variantFromListById[r.entityId]?.schemaName;
      }
      label = name ? `Asset named ${name}` : "Asset (name not found)";
    } else if (r.entityKind === "View") {
      const name = viewStore.viewsById[r.entityId]?.name;
      label = name ? `View named ${name}` : "View (name not found)";
    }

    groups.push({
      key: r.entityId,
      label,
      votes,
      satisfied: r.isSatisfied,
      requiredCount: r.requiredCount,
    });
  });
  return groups;
});

const satisfied = computed(
  () =>
    featureFlagsStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL &&
    !props.approvalData?.requirements.some((r) => r.isSatisfied === false),
);

const myVote = computed(() =>
  props.approvalData?.latestApprovals.find(
    (a) => a.isValid && a.userId === authStore.user?.pk,
  ),
);

const iApproved = computed(() => myVote.value?.status === "Approved");

const iRejected = computed(() => myVote.value?.status === "Rejected");

const mode = computed(() => {
  if (satisfied.value === true) return "approved";
  switch (props.changeSet.status) {
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

const requesterIsYou = computed(
  () => props.changeSet.mergeRequestedByUserId === authStore.user?.pk,
);
const userIsApprover = computed(() => {
  if (authStore.user && props.approvalData)
    return approverForChangeSet(authStore.user.pk, props.approvalData);
  return false;
});

const approverEmail = computed(() => props.changeSet.reviewedByUser);
const requesterEmail = computed(() => props.changeSet.mergeRequestedByUser);

const approveDate = computed(() => props.changeSet.reviewedAt as IsoDateString);
const requestDate = computed(
  () => props.changeSet.mergeRequestedAt as IsoDateString,
);

const modalData = computed(() => {
  if (mode.value === "requested") {
    return {
      title: `Approval Requested by ${
        requesterIsYou.value ? "You" : requesterEmail.value
      }`,
      date: requestDate.value,
      messageTone: "warning" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
    // approved & rejected are deprecating with the new approach
  } else if (mode.value === "approved") {
    return {
      title: approverEmail.value
        ? `Approval Granted by ${approverEmail.value}`
        : "Approval Granted",
      date: approveDate.value,
      messageTone: "success" as Tones,
      messageIcon: "check-circle" as IconNames,
    };
  } else if (mode.value === "rejected") {
    return {
      title: `Approval Rejected by ${approverEmail.value}`,
      date: approveDate.value,
      messageTone: "destructive" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  }

  return {
    title: "ERROR! Go back to HEAD",
    date: new Date(),
    messageTone: "destructive" as Tones,
  };
});

const approve = () => {
  changeSetsStore.APPROVE_CHANGE_SET_FOR_APPLY();
};

const apply = () => {
  if (authStore.user) {
    applyingChangeSet.value = true;
    changeSetsStore.APPLY_CHANGE_SET(authStore.user.name);
  }
};

const withdraw = () => {
  if (mode.value === "rejected") changeSetsStore.REOPEN_CHANGE_SET();
  else changeSetsStore.CANCEL_APPROVAL_REQUEST();
};

const rejectHandler = () => {
  changeSetsStore.REJECT_CHANGE_SET_APPLY();
};
</script>
