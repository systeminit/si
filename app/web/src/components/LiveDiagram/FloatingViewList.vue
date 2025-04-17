<template>
  <FloatingPanel
    title="Views"
    :isOpen="isOpen"
    :width="350"
    :height="600"
    position="top-left"
    @close="emit('close')"
  >
    <div class="p-2">
      <!-- Simplified version of ViewList, adapted from LeftPanelDrawer -->
      <div class="flex items-center justify-between mb-3 px-2">
        <div class="text-sm font-medium flex items-center gap-2">
          Views
          <PillCounter
            :count="viewCount"
            hideIfZero
            :paddingX="viewCount < 10 ? 'xs' : '2xs'"
          />
        </div>
        <IconButton
          icon="plus"
          size="sm"
          tooltip="Create a new View"
          @click="newView"
        />
      </div>

      <SiSearch
        ref="searchRef"
        placeholder="search views"
        @search="onSearchUpdated"
      />

      <div class="mt-3">
        <ViewCard
          v-for="view in sortedViews"
          :key="view.id"
          :view="view"
          @click="selectView(view.id)"
          @right-click="onRightClick"
        />
      </div>
    </div>
  </FloatingPanel>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { IconButton, PillCounter, SiSearch } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useViewsStore } from "@/store/views.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ViewCard from "../ViewCard.vue";
import FloatingPanel from "./FloatingPanel.vue";

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const viewsStore = useViewsStore();
const changeSetsStore = useChangeSetsStore();
const router = useRouter();

const searchRef = ref();
const searchQuery = ref("");

// Views data
const views = computed(() => Object.values(viewsStore.viewsById || {}));

const viewCount = computed(() => views.value.length);

const sortedViews = computed(() => {
  let filtered = views.value;

  // Filter by search query if provided
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    filtered = filtered.filter((view) =>
      view.name.toLowerCase().includes(query),
    );
  }

  // Sort alphabetically
  return filtered.sort((a, b) => a.name.localeCompare(b.name));
});

function onSearchUpdated(query: string) {
  searchQuery.value = query;
}

function selectView(viewId: string) {
  const changeSetId = changeSetsStore.selectedChangeSetId;

  if (changeSetId) {
    router.push({
      name: "workspaceWithViewId",
      params: {
        changeSetId,
        viewId,
      },
    });
  }

  emit("close");
}

function newView() {
  viewsStore.createNewView();
  emit("close");
}

function onRightClick(e: MouseEvent, view: any) {
  // Right-click handler if needed
}
</script>
