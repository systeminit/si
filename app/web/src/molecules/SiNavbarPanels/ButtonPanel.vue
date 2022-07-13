<template>
  <div class="flex items-center">
    <NavbarButton
      v-slot="{ hovered, open }"
      tooltip-text="Zoom"
      :options="zoomOptions"
      :text-mode="true"
      :selected="selectedButton === SelectableButton.Zoom"
      @click="changedSelectableButton(SelectableButton.Zoom)"
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
      :selected="selectedButton === SelectableButton.Theme"
      @click="changedSelectableButton(SelectableButton.Theme)"
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
import { refFrom } from "vuse-rx";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import SiArrow from "@/atoms/SiArrow.vue";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

enum SelectableButton {
  Zoom,
  Theme,
}
const selectedButton = refFrom<SelectableButton | "">("");
const changedSelectableButton = (selectableButton: SelectableButton) => {
  if (selectedButton.value === "") {
    selectedButton.value = selectableButton;
  } else {
    // Flip the selection to "unset" if the same button is clicked again.
    // FIXME(nick): this is temporary until dropdown menus are implemented for selectable buttons.
    selectedButton.value = "";
  }
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
  },
  {
    text: "175%",
  },
  {
    text: "150%",
  },
  {
    text: "125%",
  },
  {
    text: "100%",
  },
  {
    text: "75%",
  },
  {
    text: "50%",
  },
  {
    text: "25%",
  },
];

const themeOptions: SiIconDropdownOption[] = [
  {
    text: "Default (system) theme",
  },
  {
    text: "Dark theme",
  },
  {
    text: "Light theme",
  },
];
</script>
