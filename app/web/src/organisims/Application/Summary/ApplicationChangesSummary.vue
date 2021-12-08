<template>
  <SummaryCard>
    <template #title>Changes</template>

    <template #content>
      <!--
      <div class="flex flex-col w-full h-full mx-1" v-if="changesData">
        <div class="flex flex-row opened-indicator">
          <div>Opened:</div>
          <div class="ml-1">{{ changesData.openChangeSetCount }}</div>
        </div>

        <div
          class="mt-1"
          v-if="
            showSelectedChangesetData &&
              changesData.currentChangeSet &&
              changeSet
          "
        >
          <div class="flex flex-row changeset-indicator">
            <div class="">{{ changeSet.name }}</div>
          </div>

          <div class="flex flex-row pl-2 edits-indicator">
            <div>Nodes:</div>

            <div class="ml-1 additions">+</div>
            <div class="additions">
              {{ changesData.currentChangeSet.newNodes }}
            </div>

            <div class="ml-1 removals">-</div>
            <div class="removals">
              {{ changesData.currentChangeSet.deletedNodes }}
            </div>

            <div class="ml-1 updates">u</div>
            <div class="updates">
              {{ changesData.currentChangeSet.modifiedNodes }}
            </div>
          </div>

          <div class="flex flex-row pl-2 edits-indicator">
            <div>Edits:</div>
            <div class="ml-1 additions">+</div>
            <div class="additions">
              {{ changesData.currentChangeset.nodeEdits }}
            </div>
          </div>
        </div>
      </div> -->
    </template>
  </SummaryCard>
</template>

<script setup lang="ts">
import SummaryCard from "@/atoms/SummaryCard.vue";

defineProps({
  showSelectedChangesetData: {
    type: Boolean,
    default: true,
  },
  applicationId: {
    type: Number,
    required: true,
  },
});
</script>

<style scoped>
.opened-indicator {
  font-size: 10px;
  font-weight: 400;
  color: #b7b9c9;
}

.changeset-indicator {
  font-size: 10px;
  font-weight: 600;
  color: #b7b9c9;
}

.edits-indicator {
  font-size: 9px;
  font-weight: 400;
  color: #b7b9c9;
}

.additions {
  color: #a6e2a5;
}

.removals {
  color: #e2a5a5;
}

.updates {
  color: #e2c8a5;
}
</style>
