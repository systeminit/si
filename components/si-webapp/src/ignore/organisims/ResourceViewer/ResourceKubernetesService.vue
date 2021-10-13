<template>
  <div class="flex flex-col w-full text-sm" v-if="data">
    <div
      class="flex flex-row pb-2"
      v-for="item in data"
      :key="clusterName(item.data)"
    >
      <div class="flex flex-col w-full">
        <div class="w-full py-1 ml-2 text-sm text-yellow-100">
          {{ clusterName(item.data) }}
        </div>
        <div
          class="py-1 ml-4 mr-4 text-white border-t-2 border-b-2 border-gray-700"
        >
          Health
        </div>
        <ResourceFieldSlot
          label="HTTP Check"
          class="mt-2 mr-4"
          v-if="hasValue(item.data, ['healthCheckUrl'])"
        >
          <a
            :href="getValue(item.data, ['healthCheckUrl'])"
            target="_blank"
            class="underline text-blueGray-300"
            rel="noopener noreferrer"
          >
            {{ getValue(item.data, ["healthCheckUrl"]) }}
          </a>
        </ResourceFieldSlot>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { Resource } from "si-entity";
import _ from "lodash";

import ResourceField from "./ResourceField.vue";
import ResourceFieldSlot from "./ResourceFieldSlot.vue";
import ResourceFieldArray, {
  ResourceFieldArrayValue,
} from "./ResourceFieldArray.vue";

export type ClusterData = {
  name: string;
  data: Record<string, any>;
}[];

export default Vue.extend({
  name: "ResourceKubernetesService",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    //ResourceField,
    ResourceFieldSlot,
    //ResourceFieldArray,
  },
  methods: {
    clusterName(data: Record<string, any>): string {
      return `${data["clusterType"]} ${data["clusterName"]}`;
    },
    getValue(data: Record<string, any>, path: string[]): any | undefined {
      return _.get(data, path);
    },
    valueEq(
      data: Record<string, any>,
      path: string[],
      toCheck: string,
    ): boolean {
      let v = _.get(data, path);
      return _.isEqual(v, toCheck);
    },
    hasValue(data: Record<string, any>, path: string[]): boolean {
      let v = _.get(data, path);
      if (v) {
        return true;
      } else {
        return false;
      }
    },
  },
  computed: {
    data(): Record<string, any>[] | null {
      if (this.resource.data) {
        // @ts-ignore
        return Object.values(this.resource.data);
      } else {
        return [];
      }
    },
  },
});
</script>
