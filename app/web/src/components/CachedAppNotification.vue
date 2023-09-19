<template>
  <Modal ref="modalRef" title="Please refresh your browser">
    <Stack>
      <RichText>
        <p>
          Looks like you might be running a cached version of this web app. For
          new features and to ensure compatibility, please refresh your browser.
        </p>
      </RichText>
      <VButton icon="refresh" @click="reloadBrowser">Refresh</VButton>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, RichText, Stack, VButton } from "@si/vue-lib/design-system";

import axios from "axios";
import { onBeforeUnmount, onMounted, ref } from "vue";

const APP_FILENAME_REGEX = /\/?assets\/index-([0-9a-z]+).js/;

const runningHash = getRunningHash();

const modalRef = ref();

async function check() {
  const manifestUrl = `${
    window.location.origin
  }/manifest.json?timestamp=${Date.now()}`;
  const res = await axios(manifestUrl, {
    headers: { "Cache-Control": "no-cache" },
  });

  try {
    if (res.status !== 200) throw new Error("server offline");
    const latestAppFileWithHash = res.data["index.html"].file;

    const latestHash = latestAppFileWithHash.match(APP_FILENAME_REGEX)?.[1];
    if (runningHash && latestHash !== runningHash) {
      modalRef.value?.open();
    }
  } catch (err) {
    stopInterval();
  }
}

function reloadBrowser() {
  window.location.reload();
}

function getRunningHash() {
  if (import.meta.env.SSR) return "";
  // look for script tag of our main entrypoint that includes a hash
  const scriptEls = document.querySelectorAll("script[src^='/assets/index-']");
  for (const scriptEl of scriptEls) {
    const matches = scriptEl.getAttribute("src")?.match(APP_FILENAME_REGEX);
    if (matches) return matches[1];
  }
}

let intervalId: number;
function stopInterval() {
  if (window && intervalId) window.clearInterval(intervalId);
}
onMounted(() => {
  if (import.meta.env.SSR) return;
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  check();
  intervalId = window.setInterval(check, 30 * 1000);
});
onBeforeUnmount(stopInterval);
</script>
