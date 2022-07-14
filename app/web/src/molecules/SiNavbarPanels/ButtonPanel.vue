<template>
  <div class="flex items-center">
    <NavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Zoom"
      :options="zoomOptions"
      :text-mode="true"
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
import { onMounted } from "vue";

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

onMounted(() => {
  if (
    localStorage.theme === "dark" ||
    (!("theme" in localStorage) &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
  ) {
    document.documentElement.classList.add("dark");
    localStorage.setItem("color-theme", "dark");
  } else {
    document.documentElement.classList.remove("dark");
    localStorage.setItem("color-theme", "light");
  }
});

const setThemeToLight = () => {
  document.documentElement.classList.remove("dark");
  localStorage.setItem("color-theme", "light");
};

const setThemeToDark = () => {
  document.documentElement.classList.add("dark");
  localStorage.setItem("color-theme", "dark");
};

const themeOptions: SiIconDropdownOption[] = [
  {
    text: "System theme",
    action: () => {
      if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        setThemeToDark();
      } else {
        setThemeToLight();
      }
      localStorage.removeItem("color-theme");
    },
  },
  {
    text: "Light theme",
    action: setThemeToLight,
  },
  {
    text: "Dark theme",
    action: setThemeToDark,
  },
];
</script>
