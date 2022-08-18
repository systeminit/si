<template>
  <div class="flex flex-row h-full w-full">
    <!-- Filter button and list of components -->
    <div class="w-72 shrink-0 border-shade-100 h-full flex flex-col">
      <!-- Filter button and its dropdown -->
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center"
      >
        Components Menu
      </span>
      <SiBarButton
        class="h-11 border-b border-shade-100"
        fill-entire-width
        tooltip-text="Filter"
      >
        <template #default="{ hovered, open }">
          <div class="flex-row flex">
            {{ filterTitle }}
            <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
          </div>
        </template>

        <template #dropdownContent>
          <SiDropdownItem
            :checked="filter === 'all'"
            @select="changeFilter('all')"
            >Show All
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
          :class="
            selectedComponent?.componentId === component.componentId
              ? 'bg-action-500'
              : 'hover:bg-black'
          "
          class="py-2 pl-4 pr-3 cursor-pointer flex justify-between items-center"
          @click="updateSelectedComponent(component)"
        >
          <span class="shrink min-w-0 truncate mr-3">
            {{ component.componentName }}
          </span>
          <StatusIndicatorIcon
            :status="iconStatus(component)"
            class="w-6 shrink-0"
          />
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
      class="flex flex-row items-center text-center flex-grow h-full bg-shade-100"
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
    return "Show All";
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
