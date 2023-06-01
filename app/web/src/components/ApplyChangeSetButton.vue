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
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, Icon, VormInput } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useFixesStore } from "@/store/fixes.store";
import type { Recommendation } from "@/store/fixes.store";

const props = defineProps<{
  recommendations: Recommendation[]
}>();

const fixesStore = useFixesStore();
const changeSetsStore = useChangeSetsStore();

const applyButtonRef = ref();

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET2");

// Applies the current change set
const applyChangeSet = async () => {
  await changeSetsStore.APPLY_CHANGE_SET2(props.recommendations);
};

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
</script>
