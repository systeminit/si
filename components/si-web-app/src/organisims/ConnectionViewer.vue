<template>
  <div class="w-full">
    <div
      class="relative flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="flex flex-row items-center text-lg title">
        <div class="">
          {{ entity.entityType }}
        </div>

        <div class="ml-2">
          {{ entity.name }}
        </div>
      </div>
    </div>
    <div class="flex flex-col">
      <ConnectionSection
        :connections="inputConnections()"
        :enableConnectionDelete="editMode"
        title="Inbound Connections"
        @delete-connection="deleteConnection"
      />

      <ConnectionSection
        :connections="outputConnections()"
        :enableConnectionDelete="editMode"
        title="Outbound Connections"
        :reversed="true"
        class="mt-8"
        @delete-connection="deleteConnection"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Connection } from "@/api/sdf/model/connection";
import { Connections } from "@/api/sdf/dal/attributeDal";
import ConnectionSection from "@/molecules/ConnectionSection.vue";
import { Entity } from "@/api/sdf/model/entity";
import {
  AttributeDal,
  IDeleteConnectionRequest,
} from "@/api/sdf/dal/attributeDal";
import { edgeDeleted$, EdgeDeleted } from "@/observables";

import {
  changeSet$,
  editMode$,
  editSession$,
  system$,
  workspace$,
} from "@/observables";

export default Vue.extend({
  name: "ConnectionViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
    connections: {
      type: Object as PropType<Connections>,
      required: true,
      default: null,
    },
  },
  components: {
    ConnectionSection,
  },
  subscriptions: function(this: any): Record<string, any> {
    return {
      editMode: editMode$,
      currentWorkspace: workspace$,
      currentSystem: system$,
      currentChangeSet: changeSet$,
      currentEditSession: editSession$,
    };
  },
  methods: {
    inputConnections(): Connection[] {
      let connections: Connection[] = [];
      if (this.connections) {
        for (let connection of this.connections.inbound) {
          connections.push(connection);
        }
      }
      return connections;
    },
    outputConnections(): Connection[] {
      let connections: Connection[] = [];
      if (this.connections) {
        for (let connection of this.connections.outbound) {
          connections.push(connection);
        }
      }
      return connections;
    },
    async deleteConnection(edgeId: string) {
      // @ts-ignore
      if (
        // @ts-ignore
        this.currentWorkspace &&
        // @ts-ignore
        this.editMode &&
        // @ts-ignore
        this.currentChangeSet &&
        // @ts-ignore
        this.currentEditSession
      ) {
        let request: IDeleteConnectionRequest = {
          // @ts-ignore
          workspaceId: this.currentWorkspace.id,
          // @ts-ignore
          changeSetId: this.currentChangeSet.id,
          // @ts-ignore
          editSessionId: this.currentEditSession.id,
          edgeId: edgeId,
        };

        await AttributeDal.deleteConnection(request);
        edgeDeleted$.next({ edgeId: edgeId });
      }
    },
  },
});
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}

.title {
  color: #e0e0e0;
}

.sub-title {
  color: #cccccc;
}
</style>
