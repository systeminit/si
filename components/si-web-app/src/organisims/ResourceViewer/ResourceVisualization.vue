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
        <ResourceDockerImage
          :resource="resource"
          v-if="resource.entityType == 'dockerImage'"
        />
        <ResourceK8sNamespace
          :resource="resource"
          v-else-if="resource.entityType == 'k8sNamespace'"
        />
        <ResourceK8sDeployment
          :resource="resource"
          v-else-if="resource.entityType == 'k8sDeployment'"
        />
        <ResourceK8sService
          :resource="resource"
          v-else-if="resource.entityType == 'k8sService'"
        />
        <ResourceKubernetesService
          :resource="resource"
          v-else-if="
            resource.entityType == 'kubernetesService' ||
              resource.entityType == 'service'
          "
        />
        <ResourceKubernetesCluster
          :resource="resource"
          v-else-if="
            resource.entityType == 'kubernetesCluster' ||
              resource.entityType == 'awsEks' ||
              resource.entityType == 'awsEksCluster' ||
              resource.entityType == 'azureAks' ||
              resource.entityType == 'azureAksCluster'
          "
        />

        <div class="flex flex-col" v-if="Object.keys(resource.data).length > 0">
          <div class="flex flex-row justify-end py-1">
            <button
              @click="toggleRawData()"
              class="flex flex-row items-center mr-2 focus:outline-none"
            >
              <div class="flex text-xs text-gray-500">
                Raw Data
              </div>
              <ChevronDownIcon
                v-if="isRawDataExpanded"
                size="1.1x"
                class="ml-1 text-gray-500"
              />
              <ChevronRightIcon size="1.1x" v-else class="ml-1 text-gray-500" />
            </button>
          </div>
          <div class="w-full" v-if="isRawDataExpanded">
            <div
              class="mx-4 my-4 overflow-auto select-text json-code scrollbar"
            >
              <VueJsonPretty :data="resource.data" :showDoubleQuotes="false" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Resource, ResourceHealth } from "@/api/sdf/model/resource";
import ResourceDockerImage from "./ResourceDockerImage.vue";
import ResourceK8sNamespace from "./ResourceK8sNamespace.vue";
import ResourceK8sDeployment from "./ResourceK8sDeployment.vue";
import ResourceK8sService from "./ResourceK8sService.vue";
import ResourceKubernetesService from "./ResourceKubernetesService.vue";
import ResourceKubernetesCluster from "./ResourceKubernetesCluster.vue";
// import CodeMirror from "@/molecules/CodeMirror.vue";
import VueJsonPretty from "vue-json-pretty";

import {
  HeartIcon,
  ChevronRightIcon,
  ChevronDownIcon,
} from "vue-feather-icons";

interface IData {
  isDataExpanded: boolean;
  isRawDataExpanded: boolean;
}

export default Vue.extend({
  name: "ResourceVisualization",
  components: {
    // CodeMirror,
    VueJsonPretty,
    HeartIcon,
    ChevronRightIcon,
    ChevronDownIcon,
    ResourceDockerImage,
    ResourceK8sNamespace,
    ResourceK8sDeployment,
    ResourceK8sService,
    ResourceKubernetesService,
    ResourceKubernetesCluster,
  },
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  data(): IData {
    return {
      isDataExpanded: true,
      isRawDataExpanded: false,
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
    toggleRawData() {
      this.isRawDataExpanded = !this.isRawDataExpanded;
    },
    whenCodeMirrorIsVisible(): Record<string, any> {
      let style: Record<string, any> = {};
      if (this.isDataExpanded) {
        style["h-full"] = true;
      }
      return style;
    },
  },
  watch: {
    resource() {
      this.isDataExpanded = true;
      this.isRawDataExpanded = false;
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

.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}
</style>
