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
          Status
        </div>
        <ResourceFieldSlot
          label="LoadBalancer"
          class="mt-2 mr-4"
          v-if="hasValue(item.data, ['status', 'loadBalancer'])"
        >
          <div
            class="flex flex-col w-full py-2 pr-4 border border-gray-700"
            v-for="(ingress, index) in getValue(item.data, [
              'status',
              'loadBalancer',
              'ingress',
            ])"
            :key="index"
          >
            <ResourceField
              label="Hostname"
              :value="getValue(ingress, ['hostname'])"
            />
          </div>
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

export type ClusterData = {
  name: string;
  data: Record<string, any>;
}[];

export default Vue.extend({
  name: "ResourceK8sIngress",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    ResourceField,
    ResourceFieldSlot,
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
