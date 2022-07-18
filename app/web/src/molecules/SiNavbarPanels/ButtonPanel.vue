<template>
  <div class="flex items-center h-full">
    <NavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Zoom"
      :options="zoomOptions"
      :text-mode="true"
      dropdown-classes="text-center"
    >
      <div class="flex-row flex text-white">
        100%
        <SiArrow :nudge="hovered || open" class="ml-1 w-4 text-white" />
      </div>
    </NavbarButton>

    <NavbarButton tooltip-text="Copy link" @click="copyURL">
      <LinkIcon class="w-6 text-white" />
    </NavbarButton>

    <SiThemeSwitcher />

    <SiProfile />
  </div>
</template>

<script setup lang="ts">
import { LinkIcon } from "@heroicons/vue/outline";
import SiProfile from "@/molecules/SiProfile.vue";
import NavbarButton from "@/molecules/SiNavbarButtons/NavbarButton.vue";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import SiArrow from "@/atoms/SiArrow.vue";
import SiThemeSwitcher from "@/organisims/SiThemeSwitcher.vue";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
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
</script>
