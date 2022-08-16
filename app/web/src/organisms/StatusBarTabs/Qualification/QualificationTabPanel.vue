<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <div
      class="w-80 border-r-[1px] border-black text-center h-full flex flex-col"
    >
      <!-- Filter button and its dropdown -->
      <SiBarButton
        class="h-10 border-b-[1px] border-black"
        dropdown-classes="top-1 left-4"
        tooltip-text="Filter"
        fill-entire-width
      >
        <template #default="{ hovered, open }">
          <div class="flex-row flex justify-center">
            {{ filterTitle }}
            <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
          </div>
        </template>

        <template #dropdownContent>
          <SiDropdownItem
            :checked="filter === 'all'"
            @select="changeFilter('all')"
            >All
          </SiDropdownItem>
          <SiDropdownItem
            :checked="filter === 'success'"
            @select="changeFilter('success')"
            >Success
          </SiDropdownItem>
          <SiDropdownItem
            :checked="filter === 'failure'"
            @select="changeFilter('failure')"
            >Failure
          </SiDropdownItem>
        </template>
      </SiBarButton>

      <!-- List of components -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="component in list"
          :key="component.componentId"
          class="flex flex-col text-sm"
        >
          <div
            :class="
              selectedComponent?.componentId === component.componentId
                ? 'bg-action-500'
                : 'hover:bg-black'
            "
            class="py-2 truncate cursor-pointer flex flex-row justify-between"
            @click="updateSelectedComponent(component)"
          >
            <div class="text-left text-ellipsis ml-2.5 mr-6">
              {{ component.componentName }}
            </div>
            <StatusIndicatorIcon
              :status="iconStatus(component)"
              class="w-6 mr-2.5 ml-6 text-right"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Selected component view -->
    <QualificationViewerMultiple
      v-if="selectedComponent"
      :component-id="selectedComponent.componentId"
      :component-name="selectedComponent.componentName"
      :component-qualification-status="iconStatus(selectedComponent)"
    />
    <div
      v-else
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  GetSummaryResponse,
  QualificationSummaryForComponent,
} from "@/service/qualification/get_summary";
import { refFrom } from "vuse-rx";
import { QualificationService } from "@/service/qualification";
import { computed, ref } from "vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import QualificationViewerMultiple from "@/organisms/StatusBarTabs/Qualification/QualificationViewerMultiple.vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";

// Loads data for qualifications - total, succeeded, failed
const qualificationSummary = refFrom<GetSummaryResponse | undefined>(
  QualificationService.getSummary(),
);

const selectedComponent = ref<QualificationSummaryForComponent>();
const updateSelectedComponent = (
  component: QualificationSummaryForComponent,
) => {
  selectedComponent.value = component;
};

const filter = ref<"all" | "success" | "failure">("all");
const changeFilter = (newFilter: "all" | "success" | "failure") => {
  filter.value = newFilter;
};

const filterTitle = computed(() => {
  if (filter.value === "all") {
    return "All";
  } else if (filter.value === "success") {
    return "Success";
  }
  return "Failure";
});

const list = computed(() => {
  if (qualificationSummary.value === undefined) return [];
  const c = qualificationSummary.value.components;
  if (filter.value === "success") return c.filter((c) => c.failed === 0);
  else if (filter.value === "failure") return c.filter((c) => c.failed > 0);
  return c;
});

const iconStatus = (component: QualificationSummaryForComponent): Status =>
  component.succeeded === component.total
    ? "success"
    : component.failed + component.succeeded === component.total
    ? "failure"
    : "loading";
</script>
