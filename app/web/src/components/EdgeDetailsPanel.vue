<template>
  <div v-if="selectedEdge" class="flex flex-col h-full w-full overflow-hidden">
    <div class="p-xs border-b dark:border-neutral-600 flex-none">
      <Inline alignY="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Connection Details</div>
      </Inline>
    </div>

    <div class="overflow-y-auto">
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

      <template v-if="selectedEdge.changeStatus === 'deleted'">
        <Stack class="p-sm">
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
    </div>
    <!-- <template v-else>
      <div class="p-sm">
        <VButton
          tone="destructive"
          variant="ghost"
          icon="trash"
          label="Delete edge"
          @click="emit('delete')"
        />
      </div>
    </template> -->
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import {
  Icon,
  VButton,
  Inline,
  Stack,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import EdgeCard from "./EdgeCard.vue";

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const componentsStore = useComponentsStore();

const selectedEdge = computed(() => componentsStore.selectedEdge);
</script>
