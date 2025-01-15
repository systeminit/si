<template>
  <div
    :class="
      clsx(
        'relative', // needed for <ScrollArea> absolute inset-0
        themeClasses(
          'bg-shade-0 border-neutral-300',
          'bg-neutral-800 border-neutral-600',
        ),
        'w-[230px] z-[25] border-r-[3px]',
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
  themeClasses,
} from "@si/vue-lib/design-system";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { useViewsStore } from "@/store/views.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import ViewCard from "./ViewCard.vue";

const props = defineProps<{ changeSetId: ChangeSetId | undefined }>();

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
    // creating a view will force a changeset if you're on head
    const resp = await viewStore.CREATE_VIEW(viewName.value);
    // this component will unmount when we get pushed to the new changeset
    // however, it will still execute the response
    if (resp.result.success) {
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
