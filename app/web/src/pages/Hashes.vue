<template>
  <div>
    <pre>commit: {{ commitHash }}</pre>
    <pre>shared worker: {{ sharedWorkerHash }} </pre>
    <pre>tab worker: {{ webWorkerHash }} </pre>
  </div>
</template>

<script lang="ts" setup>
const commitHash = __COMMIT_HASH__;
const sharedWorkerHash = __SHARED_WORKER_HASH__;
const webWorkerHash = __WEBWORKER_HASH__;
</script>
