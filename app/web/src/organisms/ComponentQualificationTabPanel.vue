<template>
  <div class="flex flex-row h-full w-full">
    <div
      class="w-32 border-r-[1px] border-black text-center h-full flex flex-col"
    >
      <!-- Filter button and its dropdown -->
      <SiNavbarButton
        class="h-10 border-b-[1px] border-black"
        dropdown-classes="top-1 left-4"
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
      </SiNavbarButton>

      <!-- List of components -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="component in list"
          :key="component.componentId"
          class="flex flex-col text-sm"
        >
          <div
            :class="
              selectedComponentId === component.componentId
                ? 'bg-action-500'
                : 'hover:bg-black '
            "
            class="py-2"
            @click="updateSelectedComponent(component)"
          >
            {{ component.componentName }}
          </div>
        </div>
      </div>
    </div>
    <div
      v-if="selectedComponentId === undefined"
      class="flex flex-row items-center text-center w-full h-full"
    >
      <p class="w-full text-3xl text-neutral-500">No Component Selected</p>
    </div>
    <ComponentQualificationViewer
      v-else
      :component-id="selectedComponentId"
      :component-name="selectedComponentName"
    />
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
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import ComponentQualificationViewer from "@/organisms/ComponentQualificationViewer.vue";

// Loads data for qualifications - total, succeeded, failed
const qualificationSummary = refFrom<GetSummaryResponse | undefined>(
  QualificationService.getSummary(),
);

const selectedComponentId = ref<number>();
const selectedComponentName = ref<string>("");
const updateSelectedComponent = (
  component: QualificationSummaryForComponent,
) => {
  selectedComponentId.value = component.componentId;
  selectedComponentName.value = component.componentName;
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
  else return c;
});
</script>
