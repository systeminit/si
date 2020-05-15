<template>
  <div id="system-details" class="flex flex-col flex-no-wrap w-full">
    
    <div id="system-summary" class="flex-none h-40">
        {{ systemName }}
    </div>

    <div id="system-editor" class="flex h-full w-full">
        <Editor/>
    </div>

  </div>

</template>

<script>
import Editor from "@/components/views/editor/Editor.vue"

export default {
  name: 'SystemDetails',
  components: {
    Editor
  },
  data: function () {
    return {
      systemName: "demo"
    }
  }
}
</script>

<style type="text/css" scoped>

#system-summary {
  background-color: #2A2F32;
}

#system-editor {
  background-color: #000000;
}

</style>
