<template>
  <ScrollArea v-if="selectedEdge">
    <template #top>
      <SidebarSubpanelTitle label="Connection Details" icon="plug" />

      <div class="border-b dark:border-neutral-600">
        <div v-if="DEV_MODE" class="px-xs pt-xs text-2xs italic opacity-30">
          EDGE ID = {{ selectedEdge.id }}
        </div>

        <div class="p-xs">
          <EdgeCard :edgeId="selectedEdge.id" />
        </div>
        <DetailsPanelTimestamps
          :changeStatus="selectedEdge.changeStatus"
          :created="selectedEdge.createdInfo"
          :deleted="selectedEdge.deletedInfo"
        />
      </div>
    </template>

    <template v-if="selectedEdge.changeStatus === 'deleted'">
      <Stack class="px-xs py-sm">
        <ErrorMessage icon="alert-triangle" tone="warning">
          This edge will be removed from your model when this change set is
          merged
        </ErrorMessage>
        <VButton
          tone="shade"
          variant="ghost"
          size="md"
          icon="trash-restore"
          label="Restore edge"
          @click="emit('restore')"
        />
      </Stack>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import {
  VButton,
  Stack,
  ErrorMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import EdgeCard from "./EdgeCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const componentsStore = useComponentsStore();

const selectedEdge = computed(() => componentsStore.selectedEdge);
</script>
