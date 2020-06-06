<template>
<div>
<!--   <button class="bg-gray-700 px-1 py-1 border border-red-500 text-white hover:bg-teal-600 m-1 w-12"><slot/></Button> -->
  <vue-json-pretty
    class="text-white"
    :path="'res'"
    :data="data">
  </vue-json-pretty>
</div>
</template>

<script>
import VueJsonPretty from "vue-json-pretty"
export default {
  name: 'Button',
  props: [
  'type',
  'data'
  ],
  components: {
    VueJsonPretty,
  }
}
</script>