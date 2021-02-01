<template>
  <div>
    <div v-if="menuStyle === 'right'">
      <DropdownRightSided :default="optionDefault" :options="optionList" />
    </div>

    <div v-else-if="menuStyle === 'left'">
      <DropdownLeftSided :default="optionDefault" :options="optionList" />
    </div>

    <div v-else-if="menuStyle === 'standard'">
      <DropdownStandard :default="optionDefault" :options="optionList" />
    </div>

    <div v-else-if="menuStyle === 'standard-rs'">
      <DropdownStandardRightSided
        :default="optionDefault"
        :options="optionList"
      />
    </div>
  </div>
</template>

<script>
import DropdownRightSided from "./DropdownRightSided.vue";
import DropdownLeftSided from "./DropdownLeftSided.vue";
import DropdownStandard from "./DropdownStandard.vue";
import DropdownStandardRightSided from "./DropdownStandardRightSided.vue";

export default {
  name: "Dropdown",
  components: {
    DropdownLeftSided,
    DropdownRightSided,
    DropdownStandard,
    DropdownStandardRightSided,
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
    menuStyle: {
      type: String,
      default: "right",
    },
  },
};
</script>
