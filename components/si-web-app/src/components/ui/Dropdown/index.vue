<template>
  <div>
    <div v-if="optionListOrientation === 'right'">
      <DropdownRightSided :default="optionDefault" :options="optionList"/>
    </div>

    <div v-else-if="optionListOrientation === 'left'">
      <DropdownLeftSided :default="optionDefault" :options="optionList"/>
    </div>
  </div>
</template>

<script>
import DropdownRightSided from "./DropdownRightSided.vue";
import DropdownLeftSided from "./DropdownLeftSided.vue";

export default {
  name: "Dropdown",
  components: {
    DropdownLeftSided,
    DropdownRightSided
  },
  props: {
    optionDefault: {
      type: String,
      required: true,
    },
    optionList: {
      type: Array,
      required: true,
    },
    optionListOrientation: {
      type: String,
      default: "right"

    },
  },
};
</script>