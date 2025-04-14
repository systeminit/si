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
              <Icon v-if="showSpinner" name="loader" />
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
          v-for="view in sortedViews"
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
      @save="() => (ffStore.FRONTEND_ARCH_VIEWS ? bifrostCreate() : create())"
    >
      <VormInput
        ref="labelRef"
        v-model="viewName"
        required
        label="View Name"
        @enterPressed="
          () => (ffStore.FRONTEND_ARCH_VIEWS ? bifrostCreate() : create())
        "
      />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import clsx from "clsx";
import {
  Icon,
  PillCounter,
  SiSearch,
  IconButton,
  Modal,
  VormInput,
  ScrollArea,
  themeClasses,
} from "@si/vue-lib/design-system";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useViewsStore } from "@/store/views.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { bifrost, makeKey, makeArgs } from "@/store/realtime/heimdall";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ViewDescription } from "@/api/sdf/dal/views";
import { BifrostView, BifrostViewList } from "@/workers/types/dbinterface";
import ViewCard from "./ViewCard.vue";

const props = defineProps<{ changeSetId: ChangeSetId | undefined }>();

const viewStore = useViewsStore();
const ffStore = useFeatureFlagsStore();

const emit = defineEmits<{
  (e: "closed"): void;
}>();

const viewCount = computed(() => viewStore.viewList.length);

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchTerm = ref("");

const onSearchUpdated = (q: string) => {
  searchTerm.value = q;
};

const queryClient = useQueryClient();
const queryKey = makeKey("ViewList");
const viewListOverBifrost = useQuery<BifrostViewList | null>({
  queryKey,
  queryFn: async () => await bifrost<BifrostViewList>(makeArgs("ViewList")),
});
const viewAddMutation = useMutation({
  mutationFn: async (newName: string) => {
    return viewStore.CREATE_VIEW(newName);
  },
  onMutate: async (newName: string) => {
    const previousData = queryClient.getQueryData(queryKey);
    queryClient.setQueryData(queryKey, (old: BifrostViewList | null) => {
      // optimistic update code would be generated
      if (!old) {
        old = {
          id: _.uniqueId("new list id"),
          views: [],
        };
      }
      if (!old.views || !Array.isArray(old.views)) {
        old.views = [];
      }
      old.views = [
        ...old.views,
        {
          id: _.uniqueId("new-view-id"),
          name: newName,
          isDefault: false,
          created_at: new Date().toLocaleString(),
          updated_at: new Date().toLocaleString(),
        },
      ];
      return ref(old);
    });
    return { previousData };
  },
  onError: (err, _newName, context) => {
    reportError(err);
    queryClient.setQueryData(queryKey, context?.previousData);
    labelRef.value?.setError(
      `${viewName.value} is already in use. Please choose another name`,
    );
  },
  onSuccess: () => {
    modalRef.value?.close();
    viewName.value = "";
  },
});
const showSpinner = computed(
  () =>
    ffStore.FRONTEND_ARCH_VIEWS &&
    (viewListOverBifrost.isLoading.value ||
      viewListOverBifrost.fetchStatus.value === "fetching" ||
      viewAddMutation.isPending.value),
);
const bifrostCreate = () => {
  if (!viewName.value) {
    labelRef.value?.setError("Name is required");
  } else {
    viewAddMutation.mutate(viewName.value);
  }
};

const filteredViews = computed<ViewDescription[] | BifrostView[]>(() => {
  if (ffStore.FRONTEND_ARCH_VIEWS) {
    let data: BifrostView[] = [];
    if (viewListOverBifrost.isError.value) {
      // eslint-disable-next-line @typescript-eslint/no-throw-literal
      throw viewListOverBifrost.error;
    }

    if (viewListOverBifrost.data.value)
      data = viewListOverBifrost.data.value.views;

    if (!searchTerm.value) {
      return data;
    }
    return data.filter((v) =>
      v.name.toLowerCase().includes(searchTerm.value.toLowerCase()),
    );
  } else {
    if (!searchTerm.value) return viewStore.viewList;
    return viewStore.viewList.filter((v) =>
      v.name.toLowerCase().includes(searchTerm.value.toLowerCase()),
    );
  }
});

const sortedViews = computed<ViewDescription[] | BifrostView[]>(() => {
  return [...filteredViews.value].sort((a, b) => {
    if (a.isDefault) return -1;
    if (b.isDefault) return 1;
    return a.name.localeCompare(b.name);
  });
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
