<template>
  <div class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0">
    <section class="px-sm pb-sm">
      <VormInput type="container">
        <VButton
          ref="applyButtonRef"
          icon="tools"
          class="w-full"
          size="md"
          tone="success"
          loading-text="Applying Changes"
          label="Apply Changes"
          :request-status="applyChangeSetReqStatus"
          :disabled="statusStoreUpdating"
          @click="applyChangeSet"
        />
      </VormInput>
    </section>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, VormInput } from "@si/vue-lib/design-system";
import { useRouter, useRoute } from "vue-router";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import JSConfetti from "js-confetti";
import type { Recommendation } from "@/store/fixes.store";

const props = defineProps<{
  recommendations: Recommendation[];
}>();

const router = useRouter();
const route = useRoute();

const changeSetsStore = useChangeSetsStore();

const applyButtonRef = ref();

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET2");

const emit = defineEmits(["applied-change-set"]);

const celebrationEmoji = [
  "🎉",
  "🎊",
  "✨",
  "🔥",
  "⚡️",
  "🥳",
  "🍻",
  "🍺",
  "🥂",
  "🍾",
];

const celebrate = ref("🎉");
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
  await changeSetsStore.APPLY_CHANGE_SET2(props.recommendations);
  emit("applied-change-set");
  await jsConfetti.addConfetti(_.sample(confettis));
  await router.replace({
    name: route.name!, // eslint-disable-line @typescript-eslint/no-non-null-assertion
    params: {
      ...route.params,
      changeSetId: "auto",
    },
  });
};

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>
