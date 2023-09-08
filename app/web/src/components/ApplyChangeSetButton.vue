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
    @click.stop="maybeOpenModal"
  >
    Apply Changes

    <!-- modal is teleported out of here, but better to leave the button as the single root node -->
    <Modal ref="createModalRef" title="Apply Change Set" class="flex-col flex">
      <span class="text-center text-sm"
        >Applying this change set may have side-effects.</span
      >
      <span class="text-center text-sm"
        >Pick which actions will be applied to the real world:</span
      >
      <li
        v-for="action in changeSetsStore.selectedChangeSet?.actions ?? []"
        :key="action.id"
      >
        <ActionSprite :action="action" @remove="removeAction(action)" />
      </li>
      <VButton
        v-if="!changeSetsStore.headSelected"
        ref="applyButtonRef"
        icon="tools"
        size="sm"
        tone="success"
        loadingText="Applying Changes"
        label="Apply Changes"
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
import { Action } from "@/api/sdf/dal/change_set";
import ActionSprite from "@/components/ActionSprite.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";

const createModalRef = ref<InstanceType<typeof Modal> | null>(null);

const maybeOpenModal = () => {
  if (!changeSetsStore.selectedChangeSet?.actions?.length) {
    applyChangeSet();
  } else {
    createModalRef.value?.open();
  }
};

const changeSetsStore = useChangeSetsStore();
const router = useRouter();
const route = useRoute();

const removeAction = (action: Action) => {
  changeSetsStore.REMOVE_ACTION(action.id);
};

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
