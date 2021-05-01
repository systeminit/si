<template>
  <SummaryCard>
    <template v-slot:title>Changes</template>

    <template v-slot:content>
      <div class="flex flex-col w-full h-full mx-1 ">
        <div class="flex flex-row opened-indicator ">
          <div>Opened:</div>
          <div class="ml-1">{{ changesData.openedChangesetCount }}</div>
        </div>

        <div class="mt-1" v-show="showSelectedChangesetData">
          <div class="flex flex-row changeset-indicator">
            <div class="">{{ changesData.currentChangeset.id }}</div>
          </div>

          <div class="flex flex-row pl-2 edits-indicator">
            <div>Nodes:</div>

            <div class="ml-1 additions">+</div>
            <div class="additions">
              {{ changesData.currentChangeset.newNodes }}
            </div>

            <div class="ml-1 removals">-</div>
            <div class="removals">
              {{ changesData.currentChangeset.deletedNodes }}
            </div>

            <div class="ml-1 updates">u</div>
            <div class="updates">
              {{ changesData.currentChangeset.modifiedNodes }}
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
      </div>
    </template>
  </SummaryCard>
</template>

<script lang="ts">
import Vue from "vue";

import { changesData, Changes } from "@/api/visualization/changesData";

import SummaryCard from "@/atoms/SummaryCard.vue";

interface IData {
  changesData: Changes;
}

export default Vue.extend({
  name: "ChangeSummary",
  components: {
    SummaryCard,
  },
  props: {
    showSelectedChangesetData: {
      type: Boolean,
      default: true,
    },
  },
  data(): IData {
    return {
      changesData: changesData,
    };
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
