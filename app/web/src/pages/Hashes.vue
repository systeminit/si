<template>
  <div>
    <pre>commit: {{ commitHash }}</pre>
    <pre>shared worker: {{ sharedWorkerHash }} </pre>
  </div>
</template>

<script lang="ts" setup>
const commitHash = __COMMIT_HASH__;
const sharedWorkerHash = __SHARED_WORKER_HASH__;
</script>
