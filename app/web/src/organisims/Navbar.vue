<template>
  <Disclosure v-slot="{ open }" as="nav" :class="bgColor">
    <div class="mx-auto px-4 sm:px-4 lg:px-4">
      <div class="flex items-center justify-between h-16">
        <!-- Left side -->
        <div class="flex-shrink-0">
          <img
            class="block lg:hidden h-8 w-auto"
            :src="SiLogoWts"
            alt="SI Logo"
          />
          <img
            class="hidden lg:block h-8 w-auto"
            :src="SiLogoWts"
            alt="SI Logo"
          />
        </div>

        <!-- Center -->
        <div
          class="flex items-center justify-center place-items-center mx-auto"
        >
          <SiNavbarButton
            tooltip-text="Compose yourself, dammit!"
            :selected="selectedMode === Mode.Compose"
            :panel-switcher="true"
            @click="changeMode(Mode.Compose)"
          >
            <CollectionIcon />
          </SiNavbarButton>

          <SiNavbarButton
            tooltip-text="Are you a thrill beaker?"
            :selected="selectedMode === Mode.Beaker"
            :panel-switcher="true"
            @click="changeMode(Mode.Beaker)"
          >
            <BeakerIcon />
          </SiNavbarButton>

          <!-- Vertical bar -->
          <div class="w-1 h-8 self-center mx-2 bg-gray-400"></div>

          <SiNavbarButton
            tooltip-text="Eye see you"
            :selected="selectedMode === Mode.Eye"
            :panel-switcher="true"
            @click="changeMode(Mode.Eye)"
          >
            <EyeIcon />
          </SiNavbarButton>

          <SiNavbarButton
            tooltip-text="Dookie, by Green Play"
            :selected="selectedMode === Mode.Play"
            :panel-switcher="true"
            @click="changeMode(Mode.Play)"
          >
            <PlayIcon />
          </SiNavbarButton>
        </div>

        <!-- Right side -->
        <div class="hidden sm:ml-6 sm:block">
          <div class="flex items-center">
            <SiNavbarButton
              tooltip-text="Zoom"
              :options="zoomOptions"
              :text-mode="true"
              :selected="selectedButton === SelectableButton.Zoom"
              @click="changedSelectableButton(SelectableButton.Zoom)"
            >
              <div class="self-center text-center">100%</div>
            </SiNavbarButton>

            <SiNavbarButton tooltip-text="Copy link" @click="copyURL">
              <LinkIcon />
            </SiNavbarButton>

            <SiNavbarButton
              tooltip-text="Change theme"
              :options="themeOptions"
              :selected="selectedButton === SelectableButton.Theme"
              @click="changedSelectableButton(SelectableButton.Theme)"
            >
              <MoonIcon />
            </SiNavbarButton>

            <SiProfile />
          </div>
        </div>

        <!-- Mobile menu button -->
        <div class="-mr-2 flex sm:hidden">
          <DisclosureButton
            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
          >
            <span class="sr-only">Open main menu</span>
            <MenuIcon v-if="!open" class="block h-6 w-6" aria-hidden="true" />
            <XIcon v-else class="block h-6 w-6" aria-hidden="true" />
          </DisclosureButton>
        </div>
      </div>
    </div>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosureButton } from "@headlessui/vue";
import {
  EyeIcon,
  MenuIcon,
  XIcon,
  MoonIcon,
  LinkIcon,
} from "@heroicons/vue/outline";
import { PlayIcon, BeakerIcon, CollectionIcon } from "@heroicons/vue/solid";
import SiProfile from "@/molecules/SiProfile.vue";
import SiLogoWts from "@/assets/images/si-logo-wts.svg";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import { refFrom } from "vuse-rx";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";

const bgColor = "bg-[#333333]";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

enum Mode {
  Compose,
  Beaker,
  Eye,
  Play,
}
const selectedMode = refFrom<Mode>(Mode.Compose);
const changeMode = (mode: Mode) => {
  selectedMode.value = mode;
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
