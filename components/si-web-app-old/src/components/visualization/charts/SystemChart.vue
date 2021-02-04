<template>
  <Tooltip>
    <svg :width="width" :height="height">
      <rect :width="width" :height="height" :style="entityStyle" />
    </svg>
    <template v-slot:tooltip>
      <div class="flex flex-col text-gray-400">
        <div class="text-sm">
          {{ system.name }}
        </div>
      </div>
    </template>
  </Tooltip>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import {
  Resource,
  ResourceStatus,
  ResourceHealth,
} from "@/api/sdf/model/resource";
import { RootStore } from "@/store";
import Tooltip from "@/components/ui/Tooltip.vue";

export default Vue.extend({
  name: "SystemChart",
  components: {
    Tooltip,
  },
  props: {
    system: {
      type: Object as () => Entity,
    },
    width: {
      type: Number,
      default: 14,
    },
    height: {
      type: Number,
      default: 14,
    },
    rgbColor: {
      type: String,
      default: "0,0,255",
    },
  },
  computed: {
    entityStyle(): string {
      return "fill:rgba(78,141,171,1)";
    },
  },
});
</script>
