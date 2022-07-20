<template>
  <div class="w-full dark:border-[#525252] border-b-2 px-2 flex flex-col">
    <!--    TODO(victor, fletcher): replace SiTextInput and the button components with the final components-->
    <div class="w-full flex flex-row items-end">
      <SiTextInput
        title="Change Set"
        placeholder="placeholder text"
        class="px-2"
      />
      <button class="bg-[#2493F9]" :class="buttonClasses">
        <ApplyIcon class="mr-1" />
        <span>Apply</span>
      </button>
    </div>
    <div class="w-full flex flex-row mt-3 mb-2">
      <button class="bg-[#27AE60] flex-grow" :class="buttonClasses">
        <CommitIcon class="mr-1" />
        <span>Commit</span>
      </button>
      <button class="bg-[#EB5757] flex-grow" :class="buttonClasses">
        <XIcon class="inline w-4 mr-0.5" />
        Discard
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiTextInput from "@/molecules/SiTextInput.vue";
import ApplyIcon from "@/atoms/CustomIcons/ApplyIcon.vue";
import CommitIcon from "@/atoms/CustomIcons/CommitIcon.vue";
import { XIcon } from "@heroicons/vue/solid";

const buttonClasses =
  "mx-2 px-2 text-xs h-7 rounded-sm font-semibold text-white whitespace-nowrap";
</script>
