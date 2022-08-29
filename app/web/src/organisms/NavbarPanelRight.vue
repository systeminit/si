<template>
  <div class="flex items-center h-full">
    <SiBarButton tooltip-text="Copy link" @click="copyURL">
      <Icon name="link" />
    </SiBarButton>

    <SiThemeSwitcher />

    <!-- FIXME(nick,theo): dropdown-classes needs to be removed in favor of the dropdown knowing whether or not it is offscreen. -->
    <SiBarButton dropdown-classes="right-2" tooltip-text="Profile">
      <template #default="{ hovered, open }">
        <div class="flex-row flex text-white items-center">
          <CheechSvg
            class="h-8 w-8 rounded-full bg-white border-black border-2"
          />
          <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
        </div>
      </template>

      <template #dropdownContent>
        <SiDropdownItem class="text-sm" @select="onLogout"
          >Logout</SiDropdownItem
        >
      </template>
    </SiBarButton>
  </div>
</template>

<script lang="ts" setup>
import { useRouter } from "vue-router";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import SiThemeSwitcher from "@/organisms/SiThemeSwitcher.vue";
import { SessionService } from "@/service/session";
import CheechSvg from "@/assets/images/cheech-and-chong.svg?component";
import Icon from "@/ui-lib/Icon.vue";

const router = useRouter();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

const onLogout = async () => {
  await SessionService.logout();
  await router.push({ name: "login" });
};
</script>
