<template>
  <div class="flex items-center h-full">
    <SiNavbarButton tooltip-text="Zoom" :text-mode="true">
      <template #default="{ hovered, open }">
        <div class="flex-row flex text-white">
          100%
          <SiArrow :nudge="hovered || open" class="ml-1 w-4 text-white" />
        </div>
      </template>

      <template #dropdownContent>
        <SiDropdownItem>200%</SiDropdownItem>
        <SiDropdownItem>150%</SiDropdownItem>
        <SiDropdownItem>100%</SiDropdownItem>
        <SiDropdownItem>50%</SiDropdownItem>
        <SiDropdownItem>25%</SiDropdownItem>
      </template>
    </SiNavbarButton>

    <SiNavbarButton tooltip-text="Copy link" @click="copyURL">
      <LinkIcon class="w-6" />
    </SiNavbarButton>

    <SiThemeSwitcher />

    <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
    <SiNavbarButton tooltip-text="Profile" dropdown-classes="right-2">
      <template #default="{ hovered, open }">
        <div class="flex-row flex text-white">
          <img
            class="h-8 w-8 rounded-full bg-white border-black border-2"
            :src="CheechSvg"
            alt="Cheech and Chong"
          />
          <SiArrow :nudge="hovered || open" class="ml-1 w-4 text-white" />
        </div>
      </template>

      <template #dropdownContent>
        <SiDropdownItem @select="onHome">Switch to old app</SiDropdownItem>
        <SiDropdownItem @select="onLogout">Logout</SiDropdownItem>
      </template>
    </SiNavbarButton>
  </div>
</template>

<script setup lang="ts">
import LinkIcon from "@/atoms/CustomIcons/LinkIcon.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import SiThemeSwitcher from "@/organisims/SiThemeSwitcher.vue";
import { SessionService } from "@/service/session";
import { useRouter } from "vue-router";
import CheechSvg from "@/assets/images/cheech-and-chong.svg";

const router = useRouter();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

const onHome = async () => {
  await router.push({ name: "home" });
};

const onLogout = async () => {
  await SessionService.logout();
  await router.push({ name: "login" });
};
</script>
