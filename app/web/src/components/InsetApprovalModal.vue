<template>
  <div
    :class="
      clsx(
        'max-w-md flex flex-col gap-sm p-sm',
        themeClasses('bg-neutral-100 border', 'bg-neutral-900'),
      )
    "
  >
    <div class="flex flex-col gap-2xs">
      <div class="font-bold">{{ modalData.title }}</div>
      <div class="text-sm italic">
        <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
      </div>
    </div>
    <ErrorMessage
      :tone="modalData.messageTone"
      :icon="modalData.messageIcon"
      variant="block"
      class="rounded"
    >
      <template v-if="mode === 'requested'">
        This change set is currently locked until the approval is accepted or
        rejected.
        <template v-if="userIsApprover"
          >You can approve or reject this change set, or you
        </template>
        <template v-else>
          {{
            `${
              requesterIsYou
                ? "You can withdraw the approval request to make more changes and then request approval again, or you"
                : "You"
            } `
          }}
        </template>
        can switch to a different change set using the dropdown at the top of
        the screen.
      </template>
      <template v-else-if="mode === 'approved' || mode === 'rejected'">
        {{ requesterIsYou ? "Your" : "The" }} request to
        <span class="font-bold">Apply</span> change set
        <span class="font-bold">{{ changeSetName }}</span> was {{ mode }} by
        <span class="font-bold">{{ approverName + " " }}</span>
        {{
          modalData.date.toDateString() === new Date().toDateString()
            ? ""
            : "on"
        }}
        <span class="font-bold">
          <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
        </span>
        <div
          v-if="!requesterIsYou && !userIsApprover && mode === 'approved'"
          class="pt-xs"
        >
          <span class="font-bold">{{ requesterName }}</span> requested this
          <span class="font-bold">Apply</span> and can merge this change set.
          You can switch to a different change set using the dropdown at the top
          of the screen.
        </div>
      </template>
      <template v-else>
        ERROR - this message should not ever show. Something has gone wrong!
      </template>
    </ErrorMessage>
    <div
      v-if="requesterIsYou || mode === 'rejected' || userIsApprover"
      class="flex flex-row gap-sm"
    >
      <VButton
        v-if="userIsApprover && mode === 'requested'"
        label="Reject Request"
        tone="destructive"
        variant="ghost"
        @click="rejectHandler"
      />
      <VButton
        :label="modalData.buttonText"
        :tone="modalData.buttonTone"
        class="grow"
        @click="confirmHandler"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  VButton,
  Timestamp,
  Tones,
  ErrorMessage,
  IconNames,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed } from "vue";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";

export type InsetApprovalModalMode = "requested" | "approved" | "rejected";

const changeSetsStore = useChangeSetsStore();
const changeSetName = computed(() => changeSetsStore.selectedChangeSet?.name);

// TODO(Wendy) - Mock data we need to replace with real data!
const mode = computed(() => "requested" as InsetApprovalModalMode);
const requesterName = computed(() => "Paul");
const requestDate = computed(() => new Date());
const requesterIsYou = computed(() => false);
const approverName = computed(() => "Wendy");
const approveDate = computed(() => new Date());
const userIsApprover = computed(() => true);
// END MOCK DATA

const modalData = computed(() => {
  if (mode.value === "requested") {
    return {
      title: `Approval Requested by ${
        requesterIsYou.value ? "You" : requesterName.value
      }`,
      date: requestDate.value,
      buttonText: userIsApprover.value
        ? "Approve Request"
        : "Withdraw Approval Request",
      buttonTone: (userIsApprover.value ? "success" : "action") as Tones,
      messageTone: "warning" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  } else if (mode.value === "approved") {
    return {
      title: `Approval Granted by ${approverName.value}`,
      date: approveDate.value,
      buttonText: "Apply Change Set",
      buttonTone: "success" as Tones,
      messageTone: "success" as Tones,
      messageIcon: "check-circle" as IconNames,
    };
  } else if (mode.value === "rejected") {
    return {
      title: `Approval Rejected by ${approverName.value}`,
      date: approveDate.value,
      buttonText: "Make Edits",
      buttonTone: "action" as Tones,
      messageTone: "destructive" as Tones,
    };
  }

  return {
    title: "ERROR!",
    date: new Date(),
    buttonText: "Error!",
    buttonTone: "destructive" as Tones,
    messageTone: "destructive" as Tones,
  };
});

const confirmHandler = () => {
  // eslint-disable-next-line no-console
  console.log(
    "TODO - write logic to handle all possible primary button press actions here!",
  );
  if (mode.value === "requested") {
    if (userIsApprover.value) {
      // TODO - this is where the logic for approving a request goes!
    } else if (requesterIsYou.value) {
      /// TODO - this is where the logic for withdrawing a request goes!
    }
  } else if (mode.value === "approved") {
    // TODO - this is where the logic for applying an approved change set goes!
  } else if (mode.value === "rejected") {
    // TODO - this is where the logic for returning the change set to open goes!
  }
};

const rejectHandler = () => {
  // eslint-disable-next-line no-console
  console.log("TODO - write logic to handle rejecting a request here!");
};
</script>
