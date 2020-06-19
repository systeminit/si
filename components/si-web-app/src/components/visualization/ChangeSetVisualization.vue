<template>
  <div class="flex flex-col">
    
    <div class="ml-1 text-sm font-bold text-gray-400">
      changes
    </div>

    <div class="flex pl-1">
      <div class="ml-1 text-xs font-bold text-gray-400">
        open:
      </div>
      <div class="ml-1 text-xs font-normal text-gray-400">
        3
      </div>
      <alert-circle-icon size="0.75x" class="ml-1 self-center text-orange-600"/>
    </div>

  </div>
</template>

<script>
import { AlertCircleIcon } from 'vue-feather-icons'
export default {
  name: "ChangeSetVisualization",
  components: {
    AlertCircleIcon
  }
}

</script>