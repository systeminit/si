<template>
  <div class="flex flex-col h-full w-full overflow-hidden">
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle label="Multiple Assets" icon="multiselect">
          <DetailsPanelMenuIcon
            :selected="menuSelected"
            @click="
              (e) => {
                emit('openMenu', e);
              }
            "
          />
        </SidebarSubpanelTitle>
      </template>

      <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
        {{ componentsStore.selectedComponents.length }} assets selected:
      </div>
      <Stack spacing="xs" class="p-xs">
        <ComponentCard
          v-for="component in componentsStore.selectedComponents"
          :key="component.def.id"
          :titleCard="false"
          :component="component"
        />
      </Stack>
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ScrollArea, Stack } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "./ComponentCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

defineProps({
  menuSelected: { type: Boolean },
});

const componentsStore = useComponentsStore();

const emit = defineEmits<{
  (e: "openMenu", mouse: MouseEvent): void;
}>();
</script>
