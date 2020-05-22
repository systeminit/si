<template>
  <div class="flex flex-row pt-2">

    <input
      class="appearance-none text-center input-bg-color border-none text-gray-400 ml-3 pl-2 h-5 w-32 text-sm leading-tight focus:outline-none"
      type="text"
      :aria-label="keyLabel.name"
      placeholder="key"
      v-model="tagKey.name"
    />

    <input
      class="appearance-none text-center input-bg-color border-none text-gray-400 ml-1 pl-2 h-5 w-32 text-sm leading-tight focus:outline-none"
      type="text"
      :aria-label="valueLabel"
      placeholder="value"
      v-model="tagValue"
    />

      <!-- v-model="model" -->
  </div>
</template>

<script>
export default {
  name: 'TagElement',
  props: [
    'keyLabel',
    'valueLabel',
    'tagKey',
    'tagValue'
  ],
}
</script>

<style scoped>
.input-bg-color {
  background-color: #25788a;
}

</style>