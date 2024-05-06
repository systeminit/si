<template>
  <ScrollArea v-if="selectedEdge">
    <template #top>
      <SidebarSubpanelTitle label="Connection Details" icon="plug">
        <DetailsPanelMenuIcon
          @click="
            (e) => {
              emit('openMenu', e);
            }
          "
        />
      </SidebarSubpanelTitle>

      <div class="border-b dark:border-neutral-600">
        <div v-if="isDevMode" class="px-xs pt-xs text-3xs italic opacity-30">
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
          @click="modelingEventBus.emit('restoreSelection')"
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
import { isDevMode } from "@/utils/debug";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import EdgeCard from "./EdgeCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const componentsStore = useComponentsStore();
const modelingEventBus = componentsStore.eventBus;

const selectedEdge = computed(() => componentsStore.selectedEdge);

const emit = defineEmits<{
  (e: "openMenu", mouse: MouseEvent): void;
}>();
</script>
