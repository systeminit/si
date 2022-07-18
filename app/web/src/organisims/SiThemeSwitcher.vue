<template>
  <SiNavbarButton
    tooltip-text="Change theme"
    :options="themeOptions"
    dropdown-classes="right-0 text-center"
  >
    <MoonIcon class="w-6 text-white" />
  </SiNavbarButton>
</template>

<script setup lang="ts">
import { ThemeService } from "@/service/theme";
import { SiSelectOption } from "@/atoms/SiSelect2/types";
import { MoonIcon } from "@heroicons/vue/outline";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";

const themeOptions: SiSelectOption[] = [
  {
    text: "System theme",
    action: ThemeService.resetToSystems,
  },
  {
    text: "Light theme",
    action: () => ThemeService.setTo("light"),
  },
  {
    text: "Dark theme",
    action: () => ThemeService.setTo("dark"),
  },
];
</script>
