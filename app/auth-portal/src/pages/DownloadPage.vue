<template>
  <div v-if="featureFlagsStore.INSTALL_PAGE" class="overflow-hidden">
    <div class="flex flex-row items-center mb-sm">
      <div class="w-xl mr-md flex-none"><SiLogo /></div>
      <div class="flex flex-col grow">
        <h1 class="text-3xl font-bold">Install System Initiative</h1>
        <p class="text-sm italic line-clamp-2">
          Install or update to v{{ selectedVersion
          }}{{ selectedVersion === versions[0] ? " (latest)" : "" }} of System
          Initiative to get started.
        </p>
        <!-- TODO(wendy) - add "latest version" text conditional -->
      </div>
      <div v-if="versions.length > 1" class="w-40 flex-none">
        <VormInput
          v-model="selectedVersion"
          noLabel
          type="dropdown"
          :options="versions"
        />
      </div>
    </div>
    <div class="border border-neutral-400 rounded-lg p-sm">
      <h1 class="font-bold text-2xl pb-xs">Operating System</h1>
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
      <div class="border-b border-neutral-400 mb-md mt-sm" />
      <div class="font-bold text-xl">Requirements</div>
      <RichText class="py-sm">
        Before installing System Initiative, you will need to have
        <code>docker</code> installed. We suggest using
        <a
          class="font-bold dark:text-action-400 text-action-600"
          href="https://www.docker.com/products/docker-desktop/"
          target="_blank"
          >Docker Desktop</a
        >
        or Docker Engine corresponding to your native architecture{{
          selectedPlatform === "Linux"
            ? " (WSL2 users can use either Docker Desktop for WSL2 or Docker Engine inside WSL2)"
            : ""
        }}.
      </RichText>
      <div class="font-bold text-xl">Shell script for installation</div>
      <p
        class="text-sm italic dark:text-neutral-400 text-neutral-600 line-clamp-2"
      >
        Copy and paste the following command into your terminal and execute it:
      </p>
      <RichText class="py-sm">
        <pre><code class="language-shell">$ curl https://auth.systeminit.com/install.sh | sh</code></pre>
      </RichText>
      <div class="font-bold text-xl">Manual Installation</div>
      <div>
        <RichText class="py-sm">
          In order to manually install System Initiative, please download one of
          the binaries below. When the binary is downloaded, you can following
          the commands as follows:
        </RichText>
        <RichText>
          1. Extract the tarball and move the `si` binary to a directory
          included in your system's <code class="language-shell">$PATH</code
          ><br />
          2. Verify that the installation works by running the CLI command:<br />
        </RichText>
        <RichText class="py-sm">
          <pre><code class="language-shell">$ si --version</code></pre>
        </RichText>
      </div>
      <template v-if="selectedPlatformAssets.length > 0">
        <div class="font-bold text-xl">
          Binary download{{ selectedPlatformAssets.length > 1 ? "s" : "" }} for
          {{ selectedPlatform }}
        </div>
        <div
          v-for="asset in selectedPlatformAssets"
          :key="asset.id"
          class="border border-neutral-400 rounded p-sm my-sm flex flex-row justify-between items-center"
        >
          <div class="flex flex-col">
            <div class="text-sm font-bold">{{ asset.name }}</div>
            <div class="italic text-sm dark:text-neutral-400 text-neutral-600">
              Version: {{ selectedVersion }}
            </div>
          </div>
          <a
            class="flex flex-row items-center text-action-500 font-bold cursor-pointer"
            :href="asset.url"
            target="_blank"
          >
            <!-- TODO(wendy) - download link goes here -->
            <div>Download</div>
            <Icon name="download" />
          </a>
        </div>
      </template>
      <div class="font-bold text-xl">Release information</div>
      <div
        class="border border-neutral-400 rounded p-sm my-sm flex flex-row justify-between items-center"
      >
        <div class="flex flex-col">
          <div class="text-sm font-bold">Changelog</div>
          <div class="italic text-sm text-neutral-400">
            Version: {{ selectedVersion }}
          </div>
        </div>
        <a
          class="flex flex-row items-center text-action-500 font-bold cursor-pointer"
          :href="`https://github.com/systeminit/si/releases/tag/${selectedVersion}`"
          target="_blank"
        >
          <!-- TODO(wendy) - changelog link goes here -->
          <div>Github</div>
          <Icon name="arrow-square-out" />
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useHead } from "@vueuse/head";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import clsx from "clsx";
import { computed, onBeforeMount, ref } from "vue";
import { Icon, VormInput, RichText } from "@si/vue-lib/design-system";
import { Asset, useGithubStore } from "@/store/github.store";
import { useFeatureFlagsStore } from "../store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();
const githubStore = useGithubStore();

useHead({ title: "Download" });

onBeforeMount(async () => {
  await githubStore.LOAD_RELEASES();
  if (githubStore.releases.length > 0) {
    selectedVersion.value = githubStore.releases[0].version;
  }
  // console.log(githubStore.releases);
});

const releasesArray = computed(() => {
  return githubStore.releases;
});

const releasesByVersion = computed(() => {
  return releasesArray.value
    .map((release) => ({ [release.version]: release }))
    .reduce((acc, val) => ({ ...acc, ...val }), {});
});

const selectedPlatform = ref("Linux");
const platforms = ["Linux", "macOS"];

const selectedVersion = ref<string>();
const versions = computed(() => {
  return releasesArray.value.map((release) => release.version);
});

const selectedPlatformAssets = computed(() => {
  const assets = [] as Asset[];

  if (!selectedVersion.value) return assets;

  releasesByVersion.value[selectedVersion.value].assets.forEach(
    (asset: Asset) => {
      if (
        selectedPlatform.value.toLowerCase() === "macos"
          ? asset.name.toLowerCase().includes("darwin")
          : asset.name.toLowerCase().includes("linux")
      ) {
        assets.push(asset);
      }
    },
  );

  return assets;
});
</script>
