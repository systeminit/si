<template>
  <div
    class="flex items-center justify-center place-items-center mx-auto h-full"
  >
    <SiNavbarButton
      tooltip-text="Compose yourself, dammit!"
      :selected="currentViewer === 'compose'"
      @click="ViewerService.setTo('compose')"
    >
      <ComposeIcon class="w-6" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Are you a thrill beaker?"
      :selected="currentViewer === 'lab'"
      @click="ViewerService.setTo('lab')"
    >
      <BeakerIcon class="w-6 text-white" />
    </SiNavbarButton>

    <!-- Vertical bar -->
    <div class="w-0.5 h-8 self-center mx-2 bg-white"></div>

    <SiNavbarButton
      tooltip-text="Eye see you"
      :selected="currentViewer === 'view'"
      @click="ViewerService.setTo('view')"
    >
      <EyeIcon class="w-6 text-white" />
    </SiNavbarButton>

    <SiNavbarButton
      tooltip-text="Dookie, by Green Play"
      :selected="currentViewer === 'runtime'"
      @click="ViewerService.setTo('runtime')"
    >
      <PlayIcon class="w-6" />
    </SiNavbarButton>
  </div>
</template>

<script setup lang="ts">
import { EyeIcon } from "@heroicons/vue/outline";
import { BeakerIcon } from "@heroicons/vue/solid";
import { refFrom } from "vuse-rx";
import ComposeIcon from "@/atoms/CustomIcons/ComposeIcon.vue";
import PlayIcon from "@/atoms/CustomIcons/PlayIcon.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import { ViewerService } from "@/service/viewer";
import { Viewer } from "@/observable/viewer";

const currentViewer = refFrom<Viewer>(ViewerService.currentViewer());
</script>
