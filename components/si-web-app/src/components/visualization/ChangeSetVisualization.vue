<template>
  <div class="flex">
    <alert-circle-icon size="1.5x" class="text-orange-600"/>
    <div class="ml-1 text-sm font-bold text-gray-400">
      changeset
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