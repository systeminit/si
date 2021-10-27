<template>
  <div class="flex flex-col w-full text-sm" v-if="data">
    <div
      class="py-1 ml-4 mr-4 text-white border-t-2 border-b-2 border-gray-700"
    >
      Cluster Nodes
    </div>
    <ResourceFieldSlot label="Nodes" v-if="hasValue(data, ['nodes', 'items'])">
      <div
        class="flex flex-col w-full py-2 mt-2 border border-gray-700"
        v-for="(node, index) in getValue(data, ['nodes', 'items'])"
        :key="index"
      >
        <div
          class="py-1 ml-6 mr-6 text-white border-t-2 border-b-2 border-gray-700"
        >
          Node Info
        </div>
        <ResourceField
          label="OS Image"
          :value="getValue(node, ['status', 'nodeInfo', 'osImage'])"
        />
        <ResourceField
          label="Architecture"
          :value="getValue(node, ['status', 'nodeInfo', 'architecture'])"
        />
        <ResourceField
          label="Kubelet Version"
          :value="getValue(node, ['status', 'nodeInfo', 'kubeletVersion'])"
        />
        <ResourceField
          label="Kube Proxy Version"
          :value="getValue(node, ['status', 'nodeInfo', 'kubeProxyVersion'])"
        />
        <ResourceField
          label="Container Runtime Version"
          :value="
            getValue(node, ['status', 'nodeInfo', 'containerRuntimeVersion'])
          "
        />
        <div
          class="py-1 ml-6 mr-6 text-white border-t-2 border-b-2 border-gray-700"
        >
          Conditions
        </div>
        <ResourceFieldSlot
          label="Conditions"
          v-if="hasValue(node, ['status', 'conditions'])"
        >
          <div
            class="flex flex-col w-full py-2 mt-2 border border-gray-700"
            v-for="(condition, index) in getValue(node, [
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
              label="Heartbeat Time"
              :value="getValue(condition, ['lastHeartbeatTime'])"
            />
            <ResourceField
              label="Transition Time"
              :value="getValue(condition, ['lastTransitionTime'])"
            />
          </div>
        </ResourceFieldSlot>
        <div
          class="py-1 ml-6 mr-6 text-white border-t-2 border-b-2 border-gray-700"
        >
          Capacity
        </div>
        <ResourceField
          label="CPU"
          :value="getValue(node, ['status', 'capacity', 'cpu'])"
        />
        <ResourceField
          label="Pods"
          :value="getValue(node, ['status', 'capacity', 'pods'])"
        />
        <ResourceField
          label="Memory"
          :value="getValue(node, ['status', 'capacity', 'memory'])"
        />
        <ResourceField
          label="Ephemeral Storage"
          :value="getValue(node, ['status', 'capacity', 'epehemeral-storage'])"
        />
        <ResourceField
          label="Attachable EBS Volumes"
          :value="
            getValue(node, ['status', 'capacity', 'attachable-volumes-aws-ebs'])
          "
        />
        <ResourceField
          label="Attachable Volumes Azure Disk"
          :value="
            getValue(node, [
              'status',
              'capacity',
              'attachable-volumes-azure-disk',
            ])
          "
        />

        <div
          class="py-1 ml-6 mr-6 text-white border-t-2 border-b-2 border-gray-700"
        >
          Allocatable
        </div>
        <ResourceField
          label="CPU"
          :value="getValue(node, ['status', 'allocatable', 'cpu'])"
        />
        <ResourceField
          label="Pods"
          :value="getValue(node, ['status', 'allocatable', 'pods'])"
        />
        <ResourceField
          label="Memory"
          :value="getValue(node, ['status', 'allocatable', 'memory'])"
        />
        <ResourceField
          label="Ephemeral Storage"
          :value="
            getValue(node, ['status', 'allocatable', 'epehemeral-storage'])
          "
        />
        <ResourceField
          label="Attachable EBS Volumes"
          :value="
            getValue(node, [
              'status',
              'allocatable',
              'attachable-volumes-aws-ebs',
            ])
          "
        />

        <div
          class="py-1 ml-6 mr-6 text-white border-t-2 border-b-2 border-gray-700"
        >
          Addresses
        </div>
        <ResourceFieldArray label="Addresses" :value="formatAddresses(node)" />
      </div>
    </ResourceFieldSlot>

    <div
      class="py-1 ml-4 mr-4 text-white border-t-2 border-b-2 border-gray-700"
    >
      Health
    </div>
    <ResourceFieldArray label="readyz" :value="readyz" noLabels />
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
  name: "ResourceKubernetesCluster",
  props: {
    resource: {
      type: Object as PropType<Resource>,
    },
  },
  components: {
    ResourceField,
    ResourceFieldSlot,
    ResourceFieldArray,
  },
  methods: {
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
    formatAddresses(node: Record<string, any>): ResourceFieldArrayValue {
      let result: ResourceFieldArrayValue = [];
      let addressList = _.get(node, ["status", "addresses"]);
      if (addressList) {
        result = _.map(addressList, a => {
          return { label: a["type"], value: a["address"] };
        });
      }
      return result;
    },
  },
  computed: {
    readyz(): ResourceFieldArrayValue {
      if (this.data && this.data["readyz"]) {
        return _.map(this.data["readyz"], l => {
          return { label: l, value: l };
        });
      } else {
        return [];
      }
    },
    data(): Record<string, any> | null {
      return this.resource.data;
    },
  },
});
</script>
