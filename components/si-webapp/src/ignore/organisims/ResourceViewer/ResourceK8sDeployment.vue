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
          Strategy
        </div>
        <ResourceField
          label="Type"
          :value="getValue(item.data, ['spec', 'strategy', 'type'])"
        />
        <ResourceField
          v-if="
            valueEq(item.data, ['spec', 'strategy', 'type'], 'RollingUpdate')
          "
          label="Max Surge"
          :value="
            getValue(item.data, [
              'spec',
              'strategy',
              'rollingUpdate',
              'maxSurge',
            ])
          "
        />
        <ResourceField
          v-if="
            valueEq(item.data, ['spec', 'strategy', 'type'], 'RollingUpdate')
          "
          label="Max Unvailable"
          :value="
            getValue(item.data, [
              'spec',
              'strategy',
              'rollingUpdate',
              'maxUnavailable',
            ])
          "
        />
        <div
          class="py-1 ml-4 mr-4 text-white border-t-2 border-b-2 border-gray-700"
        >
          Status
        </div>
        <ResourceField
          label="Replicas"
          :value="getValue(item.data, ['status', 'replicas'])"
        />
        <ResourceField
          label="Ready Replicas"
          :value="getValue(item.data, ['status', 'readyReplicas'])"
        />
        <ResourceField
          label="Updated Replicas"
          :value="getValue(item.data, ['status', 'updatedReplicas'])"
        />
        <ResourceField
          label="Available Replicas"
          :value="getValue(item.data, ['status', 'availableReplicas'])"
        />
        <ResourceFieldSlot
          label="Conditions"
          v-if="hasValue(item.data, ['status', 'conditions'])"
        >
          <div
            class="flex flex-col w-full py-2 mt-2 border border-gray-700"
            v-for="(condition, index) in getValue(item.data, [
              'status',
              'conditions',
            ])"
            :key="index"
          >
            <ResourceField
              label="Type"
              :value="getValue(condition, ['type'])"
            />
            <ResourceField
              label="Reason"
              :value="getValue(condition, ['reason'])"
            />
            <ResourceField
              label="Status"
              :value="getValue(condition, ['status'])"
            />
            <ResourceField
              label="Message"
              :value="getValue(condition, ['message'])"
            />
            <ResourceField
              label="Update Time"
              :value="getValue(condition, ['lastUpdateTime'])"
            />
            <ResourceField
              label="Transition Time"
              :value="getValue(condition, ['lastTransitionTime'])"
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
import ResourceFieldArray, {
  ResourceFieldArrayValue,
} from "./ResourceFieldArray.vue";

export type ClusterData = {
  name: string;
  data: Record<string, any>;
}[];

export default Vue.extend({
  name: "ResourceK8sDeployment",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    ResourceField,
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
