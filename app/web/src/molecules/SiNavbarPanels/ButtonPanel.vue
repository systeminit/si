<template>
  <div class="flex items-center h-full">
    <NavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Zoom"
      :options="zoomOptions"
      :text-mode="true"
      dropdown-classes="text-center"
    >
      <div class="flex-row flex" :class="buttonClasses(hovered, open)">
        100%
        <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
      </div>
    </NavbarButton>

    <NavbarButton
      v-slot="{ hovered }"
      tooltip-text="Copy link"
      @click="copyURL"
    >
      <LinkIcon class="w-6" :class="buttonClasses(hovered, false)" />
    </NavbarButton>

    <NavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Change theme"
      :options="themeOptions"
      dropdown-classes="right-0 text-center"
    >
      <MoonIcon class="w-6" :class="buttonClasses(hovered, open)" />
    </NavbarButton>

    <SiProfile />
  </div>
</template>

<script setup lang="ts">
import { MoonIcon, LinkIcon } from "@heroicons/vue/outline";
import SiProfile from "@/molecules/SiProfile.vue";
import NavbarButton from "@/molecules/SiNavbarButtons/NavbarButton.vue";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import SiArrow from "@/atoms/SiArrow.vue";
import { ThemeService } from "@/service/theme";
import { onMounted, toRef } from "vue";
import { theme$ } from "@/observable/theme";
import { tap } from "rxjs";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

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
const zoomOptions: SiIconDropdownOption[] = [
  {
    text: "200%",
    action: () => {
      console.log("200%");
    },
  },
  {
    text: "150%",
    action: () => {
      console.log("150%");
    },
  },
  {
    text: "100%",
    action: () => {
      console.log("100%");
    },
  },
  {
    text: "50%",
    action: () => {
      console.log("50%");
    },
  },
  {
    text: "25%",
    action: () => {
      console.log("25%");
    },
  },
];

theme$.pipe(
  tap((theme) => {
    console.log(theme.value);
    // if (theme === "dark") document.documentElement.classList.add("dark");
    // else document.documentElement.classList.remove("dark");
  }),
);

const themeOptions: SiIconDropdownOption[] = [
  {
    text: "System theme",
    action: ThemeService.resetToSystems,
  },
  {
    text: "Light theme",
    action: () => theme$.next({ value: "light", source: "user" }),
  },
  {
    text: "Dark theme",
    action: () => ThemeService.setTo("dark"),
  },
];
</script>
