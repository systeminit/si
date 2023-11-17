<template>
  <VButton
    v-if="!changeSetsStore.headSelected"
    ref="applyButtonRef"
    icon="tools"
    size="md"
    tone="success"
    loadingText="Applying Changes"
    :requestStatus="applyChangeSetReqStatus"
    :disabled="statusStoreUpdating"
    @click.stop="createModalRef?.open()"
  >
    Apply Changes

    <!-- modal is teleported out of here, but better to leave the button as the single root node -->
    <Modal ref="createModalRef" title="Apply Change Set">
      <template v-if="requiresVoting">
        <div
          :class="
            clsx(
              'p-sm flex flex-row gap-xs items-center',
              !appliedByYou && 'border-b dark:border-neutral-500',
            )
          "
        >
          <UserIcon :user="applyUser" />
          <div>
            <template v-if="appliedByYou">You have</template>
            <template v-else>
              <span class="italic">{{ applyUser.name }}</span> has
            </template>
            clicked the Apply Changes button to apply all of the changes in this
            change set to Head.<template v-if="appliedByYou">
              There are other users online in this change set, so they will get
              the chance to cancel your apply.
            </template>
          </div>
        </div>
        <div
          v-if="appliedByYou"
          class="flex w-full gap-xs justify-center items-center"
        >
          <VButton
            ref="applyButtonRef"
            icon="tools"
            size="sm"
            tone="success"
            loadingText="Applying Changes"
            label="Skip Approval And Apply"
            :requestStatus="applyChangeSetReqStatus"
            :disabled="statusStoreUpdating"
            @click="applyChangeSet"
          />
        </div>
      </template>
      <template v-else-if="!requiresVoting">
        <template v-if="!requiresVoting && !hasActions">
          <span class="text-center text-sm"
            >Applying this change set may have side-effects.</span
          >
          <span class="text-center text-sm mb-3"
            >Are you sure you want to apply this change set?</span
          >
        </template>
        <template v-if="!requiresVoting && hasActions">
          <span class="text-center text-sm"
            >Applying this change set may have side-effects.</span
          >
          <span class="text-center text-sm"
            >Pick which actions will be applied to the real world:</span
          >
          <li
            v-for="action in actionsStore.proposedActions"
            :key="action.actionInstanceId"
          >
            <ActionSprite
              :action="action"
              @remove="actionsStore.REMOVE_ACTION(action.actionInstanceId)"
            />
          </li>
        </template>
      </template>
      <VButton
        v-if="!changeSetsStore.headSelected && !requiresVoting"
        ref="applyButtonRef"
        icon="tools"
        size="sm"
        tone="success"
        loadingText="Applying Changes"
        :label="!hasActions ? 'Confirm' : 'Apply Changes'"
        :requestStatus="applyChangeSetReqStatus"
        :disabled="statusStoreUpdating"
        @click="applyChangeSet"
      />
    </Modal>
  </VButton>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref } from "vue";
import * as _ from "lodash-es";
import { useRouter, useRoute } from "vue-router";
import { VButton, Modal } from "@si/vue-lib/design-system";
import JSConfetti from "js-confetti";
import clsx from "clsx";
import ActionSprite from "@/components/ActionSprite.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { usePresenceStore } from "@/store/presence.store";
import { UserInfo } from "@/components/layout/navbar/Collaborators.vue";
import UserIcon from "@/components/layout/navbar/UserIcon.vue";

const createModalRef = ref<InstanceType<typeof Modal>>();

const presenceStore = usePresenceStore();

const hasActions = computed(() => actionsStore.proposedActions.length > 0);
const requiresVoting = computed(
  () => presenceStore.usersInChangeset.length > 0,
);

const changeSetsStore = useChangeSetsStore();
const actionsStore = useActionsStore();
const router = useRouter();
const route = useRoute();

const applyButtonRef = ref();

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

let jsConfetti: JSConfetti;
const confettis = [
  { emojis: ["🎉"] },
  { emojis: ["🍿"] },
  { emojis: ["🤘", "🤘🏻", "🤘🏼", "🤘🏽", "🤘🏾", "🤘🏿"] },
  { emojis: ["❤️", "🧡", "💛", "💚", "💙", "💜"] },
  { emojis: ["🍾", "🍷", "🍸", "🍹", "🍺", "🥂", "🍻"] },
  { emojis: ["🏳️‍🌈", "🏳️‍⚧️", "⚡️", "🌈", "✨", "🔥", "🇧🇷"] },
];
onMounted(() => {
  jsConfetti = new JSConfetti({
    canvas:
      (document.getElementById("confetti") as HTMLCanvasElement) || undefined,
  });
});

const appliedByYou = computed(() => true);

const applyUser = ref<UserInfo>({
  name: "cool user 666",
  color: "magenta",
  status: "active",
});

// Applies the current change set
const applyChangeSet = async () => {
  if (!route.name) return;
  await changeSetsStore.APPLY_CHANGE_SET();
  window.localStorage.setItem("applied-changes", "true");
  router.replace({
    name: route.name,
    params: {
      ...route.params,
      changeSetId: "head",
    },
  });
  await jsConfetti.addConfetti(_.sample(confettis));
};

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>

<style lang="less" scoped>
li {
  list-style-type: none;
}
</style>
