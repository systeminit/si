<template>
  <div class="flex flex-col w-full h-full">
    <div class="flex flex-col mx-4">
      <div class="flex flex-row py-1 card-title-bar card">
        <div class="flex flex-col flex-grow">
          <div class="flex flex-row mx-2 my-1">
            <div class="text-xs">
              <HeartIcon size="1.25x" :class="healthColor" />
            </div>

            <div class="ml-2 text-xs ">
              {{ resource.timestamp }}
            </div>
          </div>

          <div v-show="resource.error" class="flex-grow w-full card ">
            <div
              class="px-2 py-2 mx-2 my-2 text-xs whitespace-pre-wrap border content-error card"
            >
              {{ resource.error }}
            </div>
          </div>
        </div>

        <button
          @click="toggleData()"
          class="mr-2 focus:outline-none"
          v-show="Object.keys(resource.data).length > 0"
        >
          <ChevronDownIcon
            v-if="isDataExpanded"
            size="1.1x"
            class="text-gray-300 "
          />
          <ChevronRightIcon size="1.1x" v-else class="text-gray-300 " />
        </button>
      </div>

      <div
        class="flex flex-col justify-start h-full card"
        v-if="isDataExpanded"
      >
        <!-- <CodeMirror
              class="max-w-md"
              :value="jsonString"
              readOnly
              mode="json"
            /> -->

        <div
          v-show="Object.keys(resource.data).length > 0"
          class="w-full overflow-auto"
        >
          <div class="mx-4 my-4 json-code">
            <VueJsonPretty :data="resource.data" :showDoubleQuotes="false" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Resource, ResourceHealth } from "@/api/sdf/model/resource";
// import CodeMirror from "@/molecules/CodeMirror.vue";
import VueJsonPretty from "vue-json-pretty";

import {
  HeartIcon,
  ChevronRightIcon,
  ChevronDownIcon,
} from "vue-feather-icons";

interface IData {
  isDataExpanded: boolean;
}

export default Vue.extend({
  name: "ResourceVisualization",
  components: {
    // CodeMirror,
    VueJsonPretty,
    HeartIcon,
    ChevronRightIcon,
    ChevronDownIcon,
  },
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  data(): IData {
    return {
      isDataExpanded: false,
    };
  },
  computed: {
    jsonString(): string {
      return JSON.stringify(this.resource, null, "\t");
    },
    jsonMode(): Object {
      return {
        name: "javascript",
        json: "true",
      };
    },
    healthColor(): Record<string, any> {
      let style: Record<string, any> = {};
      if (this.resource.health == ResourceHealth.Ok) {
        style["health-ok"] = true;
      } else if (this.resource.health == ResourceHealth.Warning) {
        style["health-warning"] = true;
      } else if (this.resource.health == ResourceHealth.Error) {
        style["health-error"] = true;
      } else if (this.resource.health == ResourceHealth.Unknown) {
        style["health-unknown"] = true;
      } else {
        style["health-unknown"] = true;
      }
      return style;
    },
  },
  methods: {
    toggleData() {
      this.isDataExpanded = !this.isDataExpanded;
    },
    whenCodeMirrorIsVisible(): Record<string, any> {
      let style: Record<string, any> = {};
      if (this.isDataExpanded) {
        style["h-full"] = true;
      }
      return style;
    },
  },
});
</script>

<style scoped>
.card {
  background-color: #1f2122;
}

.card-title-bar {
  color: #cccdb1;
}

.content-error {
  color: #f08686;
  border-color: #f08686;
}

.health-ok {
  color: #86f0ad;
  @apply animate-pulse;
}

.health-warning {
  color: #f0d286;
}

.health-error {
  color: #f08686;
}

.health-unknown {
  color: #bbbbbb;
}

.json-code {
  width: 24rem;
}
</style>
