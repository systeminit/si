<template>
  <div class="w-full">
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
