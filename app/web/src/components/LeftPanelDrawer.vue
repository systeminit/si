<template>
  <div
    :class="
      clsx(
        'relative', // needed for <ScrollArea> absolute inset-0
        'w-[230px] bg-white dark:bg-neutral-800 z-[25] border-r-[3px]',
        'border-neutral-300 border-color: dark:border-neutral-600',
      )
    "
  >
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle icon="create">
          <template #label>
            <div class="flex flex-row gap-xs items-center">
              <div>Views</div>
              <PillCounter
                :count="viewCount"
                hideIfZero
                :paddingX="viewCount < 10 ? 'xs' : '2xs'"
              />
              <IconButton
                icon="plus"
                size="sm"
                tooltip="Create a new View"
                class="ml-auto"
                @click="newView"
              />
            </div>
          </template>
        </SidebarSubpanelTitle>

        <SiSearch
          ref="searchRef"
          placeholder="search views"
          @search="onSearchUpdated"
        />
      </template>

      <div>
        <ViewCard
          v-for="view in filteredViews"
          :key="view.id"
          :view="view"
          :selected="view.id === viewStore.selectedViewId"
          :outlined="view.id === viewStore.outlinerViewId"
        />
      </div>
    </ScrollArea>

    <Modal
      ref="modalRef"
      type="save"
      size="sm"
      saveLabel="Create"
      title="Create View"
      @save="create"
    >
      <VormInput
        ref="labelRef"
        v-model="viewName"
        required
        label="View Name"
        @enterPressed="create"
      />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import clsx from "clsx";
import {
  PillCounter,
  SiSearch,
  IconButton,
  Modal,
  VormInput,
  ScrollArea,
} from "@si/vue-lib/design-system";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { useViewsStore } from "@/store/views.store";
import ViewCard from "./ViewCard.vue";

const viewStore = useViewsStore();

const emit = defineEmits<{
  (e: "closed"): void;
}>();

const viewCount = computed(() => viewStore.viewList.length);

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchTerm = ref("");

const onSearchUpdated = (q: string) => {
  searchTerm.value = q;
};

const filteredViews = computed(() => {
  if (!searchTerm.value) return viewStore.viewList;
  return viewStore.viewList.filter((v) =>
    v.name.toLowerCase().includes(searchTerm.value.toLowerCase()),
  );
});

const modalRef = ref<InstanceType<typeof Modal>>();
const labelRef = ref<InstanceType<typeof VormInput>>();
const viewName = ref("");

const newView = () => {
  modalRef.value?.open();
};

const create = async () => {
  if (!viewName.value) {
    labelRef.value?.setError("Name is required");
  } else {
    const resp = await viewStore.CREATE_VIEW(viewName.value);
    if (resp.result.success) {
      viewStore.selectView(resp.result.data.id);
      modalRef.value?.close();
      viewName.value = "";
    } else if (resp.result.statusCode === 409) {
      labelRef.value?.setError(
        `${viewName.value} is already in use. Please choose another name`,
      );
    }
  }
};
</script>
