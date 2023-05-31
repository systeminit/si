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

    <Wipe ref="wipeRef">
      <template #duringWipe></template>
      <template #afterWipe>
        <div
          v-if="changeSetApplyStatus.isPending || wipeRef?.state === 'running'"
          class="gap-2 items-center flex flex-row p-xl min-w-0 w-full justify-center"
        >
          <Icon name="loader" size="2xl" />
          <span class="text-3xl italic truncate">
            Applying Change Set
            <template v-if="changeSetsStore.selectedChangeSet">
              "{{ changeSetsStore.selectedChangeSet.name }}"
            </template>
          </span>
        </div>
        <div
          v-else-if="changeSetApplyStatus.isSuccess"
          class="gap-2 items-center flex flex-col"
        >
          <span class="text-3xl">
            {{ celebrate }} Change Set Applied! {{ celebrate }}
          </span>
          <span class="text-md italic pt-sm">
            Preparing your recommendations...
          </span>
        </div>
      </template>
    </Wipe>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from "vue";
import * as _ from "lodash-es";
import JSConfetti from "js-confetti";
import { VButton, Icon, VormInput } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useFixesStore } from "@/store/fixes.store";
import Wipe from "./Wipe.vue";

const fixesStore = useFixesStore();

const wipeRef = ref<InstanceType<typeof Wipe>>();
const applyButtonRef = ref();

const changeSetsStore = useChangeSetsStore();

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET2");

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
  { emojis: ["🏳️‍🌈", "🏳️‍⚧️", "⚡️", "🌈", "✨", "🔥"] },
];
onMounted(() => {
  jsConfetti = new JSConfetti({
    canvas:
      (document.getElementById("confetti") as HTMLCanvasElement) || undefined,
  });
});

const changeSetApplyStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET2");

// Applies the current change set
const applyChangeSet = async () => {
  if (!wipeRef.value) return; // bail if the wipe doesn't exist

  // Pick a celebration emoji!
  celebrate.value = _.sample(celebrationEmoji)!; // eslint-disable-line @typescript-eslint/no-non-null-assertion

  // Run both the wipe and the change set apply in parallel
  const wipeDone = wipeRef.value.open(applyButtonRef.value.$el);

  await changeSetsStore.APPLY_CHANGE_SET2(fixesStore.recommendations);
  await wipeDone;

  // when the change set is done done, check if the change set apply was successful
  if (changeSetApplyStatus.value.isSuccess) {
    await jsConfetti.addConfetti(_.sample(confettis));
    wipeRef.value?.close();
  }
};

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>
