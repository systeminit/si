<template>
  <div v-if="featureFlagsStore.INSTALL_PAGE" class="overflow-hidden">
    <div class="flex flex-row items-center mb-sm">
      <div class="w-xl mr-md flex-none"><SiLogo /></div>
      <div class="flex flex-col grow">
        <h1 class="text-3xl font-bold">Install System Initiative</h1>
        <p class="text-sm italic line-clamp-2">
          Install or update System Initiative to get started
        </p>
        <!-- TODO(wendy) - add "latest version" text conditional -->
      </div>
      <div v-if="versionDropdownOptions.length > 1" class="w-40 flex-none">
        <VormInput
          v-model="selectedVersion"
          noLabel
          autoSelect
          type="dropdown"
          :options="versionDropdownOptions"
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

      <RichText class="pb-md">
        <h2>Requirements</h2>
        <p>
          Before installing System Initiative, you will need to have
          <code>docker</code> installed. We suggest using
          <a
            href="https://www.docker.com/products/docker-desktop/"
            target="_blank"
            >Docker Desktop</a
          >, Docker Engine corresponding to your native architecture{{
            selectedPlatform === "Linux"
              ? " (WSL2 users can use either Docker Desktop for WSL2 or Docker Engine inside WSL2)"
              : ""
          }}
          or <a href="https://podman.io/" target="_blank">Podman</a>
          (please ensure that the podman machine has a lot of available memory
          and cpu available for running System Initiative).
        </p>
      </RichText>
      <RichText class="pb-md">
        <h2 class="!mb-0">Shell script for installation</h2>
        <div
          class="text-sm italic dark:text-neutral-400 text-neutral-600 line-clamp-2"
        >
          Copy and paste the following command into your terminal and execute
          it:
        </div>

        <pre
          @mousedown="tracker.trackEvent('copy_install_script')"
        ><code class="language-shell">$ curl -sSfL https://auth.systeminit.com/install.sh | sh</code></pre>
      </RichText>

      <RichText class="pb-md">
        <h2>Manual Installation</h2>
        <p>
          In order to manually install System Initiative, please download one of
          the binaries below. When the binary is downloaded, you can following
          the commands as follows:
        </p>
        <p>
          1. Extract the tarball and move the `si` binary to a directory
          included in your system's <code class="language-shell">$PATH</code>
          <br />
          2. Verify that the installation works by running the CLI command:
        </p>
        <pre><code class="language-shell">$ si --version</code></pre>
      </RichText>

      <RichText class="pb-md">
        <h2>Quick Start</h2>

        <p>Use the si binary to get up and running quickly:</p>
        <pre><code class="language-shell">$ si start</code></pre>

        <p>
          Head over to the
          <RouterLink :to="{ name: 'tutorial' }">tutorial</RouterLink> for more
          info.
        </p>
      </RichText>

      <template
        v-if="selectedPlatformAssets && selectedPlatformAssets.length > 0"
      >
        <RichText>
          <h2>
            Binary download{{
              selectedPlatformAssets.length > 1 ? "s" : ""
            }}
            for {{ selectedPlatform }}
          </h2>
        </RichText>
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
            @mousedown="
              tracker.trackEvent('binary_download_click', {
                version: selectedVersion,
                platform: selectedPlatform,
              })
            "
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
          @mousedown="tracker.trackEvent('changelog_click')"
        >
          <!-- TODO(wendy) - changelog link goes here -->
          <div>Github</div>
          <Icon name="arrow-square-out" />
        </a>
      </div>
    </div>
    <RichText
      class="mt-sm text-center text-sm text-neutral-500 dark:text-neutral-400"
    >
      <p>
        By using System Initiative, you agree to its
        <RouterLink :to="{ name: 'legal' }" target="_blank"
          >licensing and privacy policy</RouterLink
        >
      </p>
    </RichText>
  </div>
</template>

<script setup lang="ts">
import { useHead } from "@vueuse/head";
import * as _ from "lodash-es";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import clsx from "clsx";
import { computed, onBeforeMount, ref } from "vue";
import { Icon, VormInput, RichText } from "@si/vue-lib/design-system";
import { useGithubStore } from "@/store/github.store";
import { tracker } from "@/lib/posthog";
import { useFeatureFlagsStore } from "../store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();
const githubStore = useGithubStore();

useHead({ title: "Download" });

onBeforeMount(async () => {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  githubStore.LOAD_RELEASES();
});

const selectedPlatform = ref("Linux");
if (window && window.navigator.userAgent.includes("Mac")) {
  selectedPlatform.value = "macOS";
}
const platforms = ["Linux", "macOS"];

const selectedVersion = ref<string>();
const versionDropdownOptions = computed(() =>
  githubStore.releases.map((release) => {
    const versionAndSha = release.version.split("/").pop() || "";
    const [versionOnly, _sha] = versionAndSha.split("-");
    return {
      value: release.version,
      label: versionOnly,
    };
  }),
);

const selectedPlatformAssets = computed(() => {
  if (!selectedVersion.value) return;
  const selectedVersionData =
    githubStore.releasesByVersion[selectedVersion.value];
  return _.filter(selectedVersionData.assets, (a) =>
    selectedPlatform.value.toLowerCase() === "macos"
      ? a.name.toLowerCase().includes("darwin")
      : a.name.toLowerCase().includes("linux"),
  );
});
</script>
