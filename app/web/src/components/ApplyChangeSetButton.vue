<template>
  <span>
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
      @click.stop="maybeOpenModal"
    />
    <Modal ref="createModalRef" title="Apply Change Set" class="flex-col flex">
      <span class="text-center text-sm"
        >Applying this change set may have side-effects.</span
      >
      <span class="text-center text-sm"
        >Pick which actions will be applied to the real world:</span
      >
      <li v-for="(obj, key) in fixesStore.recommendationsSelection" :key="key">
        <RecommendationSprite
          :key="key"
          :recommendation="obj.recommendation"
          :selected="obj.selected"
          @click.stop
          @toggle="toggleRecommendation($event, obj.recommendation)"
        />
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
  </span>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref } from "vue";
import * as _ from "lodash-es";
import { useRouter, useRoute } from "vue-router";
import { VButton, Modal } from "@si/vue-lib/design-system";
import JSConfetti from "js-confetti";
import RecommendationSprite from "@/components/RecommendationSprite2.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useFixesStore } from "@/store/fixes.store";
import type { Recommendation } from "@/store/fixes.store";

const createModalRef = ref<InstanceType<typeof Modal> | null>(null);

const fixesStore = useFixesStore();

const maybeOpenModal = () => {
  if (_.keys(fixesStore.recommendationsSelection).length === 0) {
    applyChangeSet();
  } else {
    createModalRef.value?.open();
  }
};

const toggleRecommendation = (
  selected: boolean,
  recommendation: Recommendation,
) => {
  const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
  fixesStore.recommendationsSelection[key] = { selected, recommendation };
};

const changeSetsStore = useChangeSetsStore();
const router = useRouter();
const route = useRoute();

const applyButtonRef = ref();

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET2");

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
  await changeSetsStore.APPLY_CHANGE_SET2(fixesStore.enabledRecommendations);
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
