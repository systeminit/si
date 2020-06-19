<template>
  <div class="flex w-full h-6 status-bar justify-end">

    <div class="flex items-center">
      <div class="text-sm font-light text-gray-400">mode:</div>
      <div class="text-sm font-normal pl-1 pr-4 mode-edit">
        edit
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "ChangeSetBar"
};
</script>

<style>
.status-bar {
  background-color: #1F2324;
}

.mode-edit {
  color: #D9A35E;
}

</style>