<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <ModeButton
      v-slot="{ hovered }"
      tooltip-text="Compose yourself, dammit!"
      :selected="selectedMode === Mode.Compose"
      :panel-switcher="true"
      @click="changeMode(Mode.Compose)"
    >
      <ComposeIcon
        class="w-6"
        :selected="selectedMode === Mode.Compose || hovered"
      />
    </ModeButton>

    <ModeButton
      v-slot="{ hovered }"
      tooltip-text="Are you a thrill beaker?"
      :selected="selectedMode === Mode.Beaker"
      :panel-switcher="true"
      @click="changeMode(Mode.Beaker)"
    >
      <BeakerIcon
        class="w-6"
        :class="buttonClasses(hovered, selectedMode === Mode.Beaker)"
      />
    </ModeButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-2 bg-white"></div>

    <ModeButton
      v-slot="{ hovered }"
      tooltip-text="Eye see you"
      :selected="selectedMode === Mode.Eye"
      :panel-switcher="true"
      @click="changeMode(Mode.Eye)"
    >
      <EyeIcon
        class="w-6"
        :class="buttonClasses(hovered, selectedMode === Mode.Eye)"
      />
    </ModeButton>

    <ModeButton
      v-slot="{ hovered }"
      tooltip-text="Dookie, by Green Play"
      :selected="selectedMode === Mode.Play"
      :panel-switcher="true"
      @click="changeMode(Mode.Play)"
    >
      <PlayIcon class="w-6" :selected="selectedMode === Mode.Play || hovered" />
    </ModeButton>
  </div>
</template>

<script setup lang="ts">
import { EyeIcon } from "@heroicons/vue/outline";
import { BeakerIcon } from "@heroicons/vue/solid";
import ModeButton from "@/molecules/SiNavbarButtons/ModeButton.vue";
import { refFrom } from "vuse-rx";
import ComposeIcon from "@/atoms/CustomIcons/ComposeIcon.vue";
import PlayIcon from "@/atoms/CustomIcons/PlayIcon.vue";

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

const buttonClasses = (
  hovered: boolean,
  selected: boolean,
): Record<string, boolean> => {
  if (hovered || selected) {
    return {
      block: true,
      "text-white": true,
    };
  }
  return {
    block: true,
    "text-gray-300": true,
  };
};
</script>
