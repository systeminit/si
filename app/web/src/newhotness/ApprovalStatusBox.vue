<template>
  <div
    v-if="mode !== 'error'"
    :class="
      clsx(
        'flex flex-col gap-sm p-sm',
        'rounded shadow-2xl overflow-hidden',
        themeClasses('bg-shade-0 border', 'bg-neutral-900'),
      )
    "
  >
    <!-- HEADER -->
    <div class="flex flex-row flex-none gap-md mb-sm items-center">
      <div class="flex flex-col">
        <TruncateWithTooltip class="font-bold italic pb-2xs">
          {{ changeSetName }}
        </TruncateWithTooltip>
        <TruncateWithTooltip class="font-bold pb-2xs">{{
          modalData.title
        }}</TruncateWithTooltip>
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
        <template v-else-if="mode === 'approved'">
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
        clsx('flex flex-row gap-xs flex-1 overflow-hidden place-content-evenly')
      "
    >
      <div
        :class="
          clsx('flex flex-col gap-xs overflow-hidden text-center basis-1/2')
        "
      >
        <RouterLink
          :to="{
            name: 'workspace-audit',
            params: { changeSetId: 'auto' },
          }"
          target="_blank"
          class="text-action-500 hover:underline pl-4 pb-2xs text-sm font-bold"
          >See the breakdown of changes
          <Icon
            size="sm"
            name="logs-pop-square"
            class="ml-2xs inline-block mb-[-.3em]"
          />
        </RouterLink>
      </div>
    </div>

    <!-- MAIN SECTION -->
    <div
      :class="
        clsx('flex flex-row gap-xs flex-1 overflow-hidden place-content-evenly')
      "
    >
      <div class="flex flex-col basis-1/2 text-sm gap-xs overflow-y-auto">
        <div
          v-for="group in requirementGroups"
          :key="group.key"
          class="border-neutral-200 dark:border-neutral-700 border"
        >
          <div class="bg-neutral-200 dark:bg-neutral-700 p-xs">
            {{ group.requiredCount }} of the following users for{{
              group.labels.length > 1
                ? ` ${group.labels.length} requirements:`
                : ""
            }}
            <span v-if="group.labels.length === 1" class="italic">{{
              group.labels[0]
            }}</span>
            <TruncateWithTooltip
              v-else
              expandOnClick
              :expandableStringArray="group.labels"
              class="italic break-all"
            />
          </div>
          <ul>
            <li
              v-for="vote in group.votes"
              :key="vote.user.id"
              :class="
                clsx(
                  'flex flex-row items-center gap-xs px-xs py-2xs',
                  themeClasses('even:bg-neutral-100', 'even:bg-neutral-800'),
                )
              "
            >
              <TruncateWithTooltip class="flex-grow"
                >{{ vote.user.name }} ({{
                  vote.user.email
                }})</TruncateWithTooltip
              >
              <div
                :class="
                  clsx(
                    'flex flex-col items-center flex-none w-[60px]',
                    vote.status ? 'font-bold' : 'italic',
                    vote.status === 'Rejected' && 'text-destructive-500',
                    vote.status === 'Approved' && 'text-success-500',
                  )
                "
              >
                <div v-if="!vote.status">Waiting...</div>
                <div v-else>{{ vote.status }}</div>
              </div>
              <span class="flex flex-row items-center flex-none">
                <Icon
                  size="md"
                  name="thumbs-up"
                  tone="success"
                  :class="clsx(vote.status !== 'Approved' ? 'opacity-20' : '')"
                />
                <Icon
                  size="md"
                  name="thumbs-down"
                  tone="error"
                  :class="clsx(vote.status !== 'Rejected' ? 'opacity-20' : '')"
                />
              </span>
            </li>
          </ul>
        </div>
      </div>
    </div>

    <!-- BUTTONS -->
    <div class="flex flex-row flex-none gap-sm justify-center mt-sm">
      <VButton
        label="Withdraw Request"
        tone="warning"
        variant="ghost"
        icon="x"
        @click="emit('withdraw', changeSet.id)"
      />
      <template v-if="userIsApprover">
        <VButton
          :disabled="iRejected"
          label="Reject Request"
          tone="destructive"
          icon="thumbs-down"
          @click="emit('reject', changeSet.id)"
        />
        <VButton
          :disabled="iApproved"
          label="Approve Request"
          tone="success"
          icon="thumbs-up"
          @click="emit('approve', changeSet.id)"
        />
      </template>
    </div>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
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
import { computed } from "vue";
import { RouterLink } from "vue-router";
import clsx from "clsx";
import { ChangeSet, ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  ApprovalData,
  ApprovalStatus,
  approverForChangeSet,
} from "./logic_composables/change_set";
import { useContext } from "./logic_composables/context";
import { User } from "@/api/sdf/dal/user";
import { WorkspaceUser } from "@/store/auth.store";

const props = defineProps<{
  approvalData: ApprovalData;
  changeSet: ChangeSet;
  mode: "approved" | "rejected" | "requested" | "error";
  user: User;
  workspaceUsers: Record<string, WorkspaceUser>;
}>();

const changeSetName = computed(() => props.changeSet.name);

interface RequirementGroup {
  key: string;
  labels: string[];
  votes: Vote[];
  satisfied: boolean;
  requiredCount: number;
}
interface Vote {
  user: WorkspaceUser;
  status?: ApprovalStatus;
}

const requirementGroups = computed(() => {
  for (const u of Object.keys(props.workspaceUsers)) {
    console.log("NICK USER", props.workspaceUsers[u]);
  }
  console.log("NICK APPROVAL", props.user, props.workspaceUsers);
  const groups: Map<Set<string>, RequirementGroup> = new Map();
  props.approvalData.requirements.forEach((r) => {
    const userIds = Object.values(r.approverGroups)
      .flat()
      .concat(r.approverIndividuals);
    const votes: Vote[] = [];
    userIds.forEach((id) => {
      const user = props.workspaceUsers[id];
      if (!user) {
        return;
      }
      const submitted = props.approvalData.latestApprovals.find(
        (a) =>
          a.isValid &&
          a.userId === id &&
          r.applicableApprovalIds.includes(a.id),
      );
      const vote: Vote = { user };
      if (submitted) vote.status = submitted.status;
      votes.push(vote);
    });

    // TODO(nick): restore different kinds of approval requirements.
    let label;
    if (r.entityKind === "ApprovalRequirementDefinition") {
      label = ["Approval Requirement change"];
      // } else if (r.entityKind === "Schema") {
      //   const variantForSchema = assetStore.schemaVariants.find(
      //     (thing) => thing.schemaId === r.entityId,
      //   );
      //   label = variantForSchema?.schemaName
      //     ? `Asset named ${variantForSchema?.schemaName}`
      //     : "an Asset";
      // } else if (r.entityKind === "SchemaVariant") {
      //   let name = assetStore.variantFromListById[r.entityId]?.displayName;
      //   if (!name) {
      //     name = assetStore.variantFromListById[r.entityId]?.schemaName;
      //   }
      //   label = name ? `Asset named ${name}` : "Asset (name not found)";
      // }
      // else if (r.entityKind === "View") {
      //   const name = views.value.find((v) => v.id === r.entityId)?.name;
      //   label = [name ? `View named ${name}` : "View (name not found)"];
    } else {
      label = ["Workspace change"];
    }

    const group: RequirementGroup = {
      key: r.entityId,
      labels: label,
      votes,
      satisfied: r.isSatisfied,
      requiredCount: r.requiredCount,
    };

    // Check if this RequirementGroup has the same votes and/or label as an existing one and group/filter accordingly
    const key = new Set(group.votes.map((vote) => vote.user.id));
    const check = [...groups.entries()].find(
      ([k, _]) => k.size === key.size && [...k].every((i) => key.has(i)),
    );
    if (check) {
      const [_, set] = check;
      const label = group.labels[0];
      if (label && !set.labels.includes(label)) {
        // different label and same votes - group together
        set.labels.push(label);
      }
      // same label and same votes - don't push in
    } else {
      groups.set(key, group);
    }
  });
  return [...groups.values()];
});

const myVote = computed(() =>
  props.approvalData.latestApprovals.find(
    (a) => a.isValid && a.userId === props.user.pk,
  ),
);

const iApproved = computed(() => myVote.value?.status === "Approved");

const iRejected = computed(() => myVote.value?.status === "Rejected");

const requesterIsYou = computed(
  () => props.changeSet.mergeRequestedByUserId === props.user.pk,
);
const userIsApprover = computed(() =>
  approverForChangeSet(props.user.pk, props.approvalData),
);

const approverEmail = computed(() => props.changeSet.reviewedByUser);
const requesterEmail = computed(() => props.changeSet.mergeRequestedByUser);

const approveDate = computed(() => props.changeSet.reviewedAt as IsoDateString);
const requestDate = computed(
  () => props.changeSet.mergeRequestedAt as IsoDateString,
);

const modalData = computed(() => {
  if (props.mode === "requested") {
    return {
      title: `Approval Requested by ${
        requesterIsYou.value ? "You" : requesterEmail.value
      }`,
      date: requestDate.value,
      messageTone: "warning" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
    // approved & rejected are deprecating with the new approach
  } else if (props.mode === "approved") {
    return {
      title: approverEmail.value
        ? `Approval Granted by ${approverEmail.value}`
        : "Approval Granted",
      date: approveDate.value,
      messageTone: "success" as Tones,
      messageIcon: "check-circle" as IconNames,
    };
  } else if (props.mode === "rejected") {
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

const emit = defineEmits<{
  (e: "approve", changeSetId: ChangeSetId): void;
  (e: "reject", changeSetId: ChangeSetId): void;
  (e: "withdraw", changeSetId: ChangeSetId): void;
}>();
</script>
