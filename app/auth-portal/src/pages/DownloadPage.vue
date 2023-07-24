<template>
  <div v-if="featureFlagsStore.INSTALL_PAGE" class="overflow-hidden">
    <div class="flex flex-row items-center mb-sm">
      <div class="w-xl mr-md flex-none"><SiLogo /></div>
      <div class="flex flex-col grow">
        <h1 class="text-2xl font-bold">Install System Initiative</h1>
        <p class="text-sm italic line-clamp-2">
          Install or update to v{{ selectedVersion }} (latest version) of System
          Initiative to get started.
        </p>
      </div>
      <div class="w-20 flex-none">
        <VormInput
          v-model="selectedVersion"
          type="dropdown"
          :options="versions"
        />
      </div>
    </div>
    <div class="border border-neutral-400 rounded-lg p-sm">
      <h1 class="font-bold text-lg">Operating System</h1>
      <div class="flex flex-row gap-2 mt-xs">
        <div
          v-for="platform in platforms"
          :key="platform"
          :class="
            clsx(
              'border border-transparent rounded p-xs cursor-pointer',
              'dark:hover:border-action-400 hover:border-action-500 dark:hover:text-action-400 hover:text-action-500',
              selectedPlatform === platform &&
                'dark:border-action-400 border-action-500 dark:text-action-400 text-action-500 dark:bg-transparent bg-action-100 font-bold',
            )
          "
          @click="selectedPlatform = platform"
        >
          {{ platform }}
        </div>
      </div>
      <div class="border-b border-neutral-400 my-md" />
      <div class="font-bold">Shell script for {{ selectedPlatform }}</div>
      <RichText class="py-sm">
        <pre><code class="language-shell">$ script goes here for {{ selectedPlatform }}</code></pre>
      </RichText>
      <div class="font-bold">Binary download for {{ selectedPlatform }}</div>
      <div
        class="border border-neutral-400 rounded p-sm my-sm flex flex-row justify-between items-center"
      >
        <div class="flex flex-col">
          <div class="text-sm font-bold">X86_64</div>
          <div class="italic text-sm text-neutral-400">
            Version: {{ selectedVersion }}
          </div>
        </div>
        <div class="flex flex-row items-center text-action-500 font-bold">
          <div>Download</div>
          <Icon name="download" />
        </div>
      </div>
      <div class="font-bold">Release information</div>
      <div
        class="border border-neutral-400 rounded p-sm my-sm flex flex-row justify-between items-center"
      >
        <div class="flex flex-col">
          <div class="text-sm font-bold">Changelog</div>
          <div class="italic text-sm text-neutral-400">
            Version: {{ selectedVersion }}
          </div>
        </div>
        <div class="flex flex-row items-center text-action-500 font-bold">
          <div>Github</div>
          <Icon name="arrow-square-out" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useHead } from "@vueuse/head";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import clsx from "clsx";
import { ref } from "vue";
import { Icon, RichText, VormInput } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "../store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();

useHead({ title: "Download" });

const selectedPlatform = ref("Linux");
const platforms = ["Linux", "macOS"];

const selectedVersion = ref("6.6.6");
const versions = ["6.6.6", "4.2.0", "1.3.12"];
</script>
