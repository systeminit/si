<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <SiNavbarButton
      tooltip-text="Compose yourself, dammit!"
      :selected="selectedMode === Mode.Compose"
      :panel-switcher="true"
      @click="changeMode(Mode.Compose)"
    >
      <ComposeIcon class="w-6 text-white" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Are you a thrill beaker?"
      :selected="selectedMode === Mode.Beaker"
      :panel-switcher="true"
      @click="changeMode(Mode.Beaker)"
    >
      <BeakerIcon class="w-6 text-white" />
    </SiNavbarButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-2 bg-white"></div>

    <SiNavbarButton
      tooltip-text="Eye see you"
      :selected="selectedMode === Mode.Eye"
      :panel-switcher="true"
      @click="changeMode(Mode.Eye)"
    >
      <EyeIcon class="w-6 text-white" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Dookie, by Green Play"
      :selected="selectedMode === Mode.Play"
      :panel-switcher="true"
      @click="changeMode(Mode.Play)"
    >
      <PlayIcon class="w-6 text-white" />
    </SiNavbarButton>
  </div>
</template>

<script setup lang="ts">
import { EyeIcon } from "@heroicons/vue/outline";
import { BeakerIcon } from "@heroicons/vue/solid";
import { refFrom } from "vuse-rx";
import ComposeIcon from "@/atoms/CustomIcons/ComposeIcon.vue";
import PlayIcon from "@/atoms/CustomIcons/PlayIcon.vue";
import SiNavbarButton from "@/atoms/SiNavbarButton.vue";

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
</script>
