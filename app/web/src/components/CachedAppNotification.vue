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
import axios from "axios";
import { onBeforeUnmount, onMounted, ref } from "vue";
import { Modal, RichText, Stack, VButton } from "@si/vue-lib/design-system";
import {
  cachedAppEmitter,
  SHOW_CACHED_APP_NOTIFICATION_EVENT,
} from "@/store/realtime/cached_app_emitter";

// const APP_FILENAME_REGEX = /\/?assets\/index-([0-9a-z]+).js/;
const getFilenameFromPath = (path: string) => path.split("/").pop();

const runningHash = getRunningHash();

const modalRef = ref();

cachedAppEmitter.on(SHOW_CACHED_APP_NOTIFICATION_EVENT, () => {
  modalRef.value?.open();
});

async function check() {
  const manifestUrl = `${
    window.location.origin
  }/manifest.json?timestamp=${Date.now()}`;

  try {
    const res = await axios(manifestUrl, {
      headers: { "Cache-Control": "no-cache" },
    });

    const latestAppFileWithHash = res.data["index.html"].file;

    const latestHash = getFilenameFromPath(latestAppFileWithHash);
    if (runningHash && latestHash !== runningHash) {
      modalRef.value?.open();
    }
  } catch (err) {
    // local dev errors here because the manifest file doesn't exist
    stopInterval();
  }
}

function reloadBrowser() {
  window.location.reload();
}

function getRunningHash() {
  if (import.meta.env.SSR) return "";
  // look for script tag of our main entrypoint that includes a hash
  const scriptEls = document.querySelectorAll("script");
  for (const scriptEl of scriptEls) {
    // const matches = scriptEl.getAttribute("src")?.match(APP_FILENAME_REGEX);
    const matches = getFilenameFromPath(scriptEl.getAttribute("src") ?? "");
    if (matches?.includes("main")) return matches;
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
