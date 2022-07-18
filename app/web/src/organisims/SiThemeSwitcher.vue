<template>
  <NavbarButton
    v-slot="{ hovered, open }"
    tooltip-text="Change theme"
    :options="themeOptions"
    dropdown-classes="right-0 text-center"
  >
    <MoonIcon class="w-6" :class="buttonClasses(hovered, open)" />
  </NavbarButton>
</template>

<script setup lang="ts">
import { ThemeService } from "@/service/theme";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import { MoonIcon } from "@heroicons/vue/outline";
import NavbarButton from "@/molecules/SiNavbarButtons/NavbarButton.vue";

const themeOptions: SiIconDropdownOption[] = [
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

const buttonClasses = (hovered: boolean, selected: boolean) => {
  if (hovered || selected) {
    return {
      block: true,
      "text-white": true,
    };
  }
  return {
    block: true,
    "text-gray-400": true,
  };
};
</script>
