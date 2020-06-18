<template>
  <div class="flex w-full h-6 changeset-status-bar justify-end">

    <div class="flex items-center">
      <div class="text-sm font-light text-gray-400">status:</div>
      <div class="text-sm font-normal pl-1 pr-6 text-orange-300">
        edit mode
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
.changeset-status-bar {
  background-color: #212324;
}
</style>