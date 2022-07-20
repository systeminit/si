<template>
  <div class="flex items-center h-full">
    <SiNavbarButton
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
    </SiNavbarButton>

    <SiNavbarButton tooltip-text="Copy link" @click="copyURL">
      <LinkIcon class="w-6 text-white" />
    </SiNavbarButton>

    <SiThemeSwitcher />

    <SiNavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Profile"
      :options="profileOptions"
      dropdown-classes="right-2"
    >
      <div class="flex-row flex text-white">
        <img
          class="h-8 w-8 rounded-full bg-white border-black border-2"
          :src="CheechSvg"
          alt="Cheech and Chong"
        />
        <SiArrow :nudge="hovered || open" class="ml-1 w-4 text-white" />
      </div>
    </SiNavbarButton>
  </div>
</template>

<script setup lang="ts">
import { LinkIcon } from "@heroicons/vue/outline";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import { SiDropdownOption } from "@/atoms/SiDropdown.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import SiThemeSwitcher from "@/organisims/SiThemeSwitcher.vue";
import { SessionService } from "@/service/session";
import { useRouter } from "vue-router";
import CheechSvg from "@/assets/images/cheech-and-chong.svg";

const router = useRouter();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

const zoomOptions: SiDropdownOption[] = [
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

const profileOptions: SiDropdownOption[] = [
  {
    text: "Switch to old app",
    action: async () => {
      await router.push({ name: "home" });
    },
  },
  {
    text: "Logout",
    action: async () => {
      await SessionService.logout();
      await router.push({ name: "login" });
    },
  },
];
</script>
