<template>
  <div ref="property-panel" class="w-full flex-col h-full w-full">
    <div id="property-panel-menu" class="flex-row flex-no-wrap content-between bg-black h-30 w-full">
     
     <ui-button ref="submitButton" kind="icon">
        <filter-icon size="1.5x" class="custom-class"></filter-icon>
      </ui-button>

      <ui-button ref="submitButton" kind="icon">
        <code-icon size="1.5x" class="custom-class"></code-icon>
      </ui-button>

      <ui-button ref="submitButton" kind="icon">
        <search-icon size="1.5x" class="custom-class"></search-icon>
      </ui-button>

      <ui-button ref="submitButton" kind="icon">
        <filter-icon size="1.5x" class="custom-class"></filter-icon>
      </ui-button>

      <ui-button ref="submitButton" kind="icon">
        <settings-icon size="1.5x" class="custom-class"></settings-icon>
      </ui-button>

      <ui-button ref="submitButton" kind="icon">
        <maximize-2-icon size="1.5x" class="custom-class"></maximize-2-icon>
      </ui-button>
      
    </div>
  </div>
</template>

<script>
import UiButton from "@/components/ui/button/UiButton.vue";
import {
  Maximize2Icon,
  SettingsIcon,
  FilterIcon,
  SearchIcon,
  CodeIcon,
} from "vue-feather-icons";

export default {
  name: "HelloWorld",
  components: {
    Maximize2Icon,
    SettingsIcon,
    FilterIcon,
    SearchIcon,
    CodeIcon,
    UiButton,
  },
};
</script>
