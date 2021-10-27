<template>
  <div class="flex flex-col w-full text-sm" v-if="data">
    <div
      class="flex flex-row pb-2"
      v-for="item in data"
      :key="clusterName(item.data)"
    >
      <div class="flex flex-col w-full">
        <div class="w-full ml-2 text-sm text-yellow-100">
          {{ clusterName(item.data) }}
        </div>
        <ResourceField
          label="Status Phase"
          :value="item.data['status']['phase']"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { Resource } from "si-entity";
import _ from "lodash";

import ResourceField from "./ResourceField.vue";
import ResourceFieldArray, {
  ResourceFieldArrayValue,
} from "./ResourceFieldArray.vue";

export type ClusterData = {
  name: string;
  data: Record<string, any>;
}[];

export default Vue.extend({
  name: "ResourceK8sNamespace",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    ResourceField,
    //ResourceFieldArray,
  },
  methods: {
    clusterName(data: Record<string, any>): string {
      return `${data["clusterType"]} ${data["clusterName"]}`;
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
      // @ts-ignore
      if (this.resource.data.inspect && this.resource.data.inspect.length > 0) {
        // @ts-ignore
        return this.resource.data.inspect[0];
      } else {
        return null;
      }
    },
  },
});
</script>
