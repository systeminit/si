<template>
  <v-container class="fluid">
    <v-card :loading="$apollo.loading">
      <v-card-title>{{ getEntity.displayName }}</v-card-title>
      <v-card-subtitle>{{ siComponent.name }}</v-card-subtitle>
      <v-card-text>
        <v-btn-toggle>
          <v-btn
            v-for="action in siComponent.showActions"
            v-bind:key="action.displayName"
            @click="runAction(getEntity.id, action.mutation)"
          >
            {{ action.displayName }}
          </v-btn>
        </v-btn-toggle>
        <v-list flat>
          <v-list-item
            v-for="property in siComponent.showProperties"
            v-bind:key="property.property"
          >
            <v-list-item-content>
              <v-list-item-title>
                {{ property.displayName }}
              </v-list-item-title>
              <div v-if="property.showAs == 'text'">
                {{ getEntity[property.property] }}
              </div>
              <v-textarea
                v-else-if="property.showAs == 'textarea'"
                no-resize
                outlined
                full-width
                flat
                single-line
                readonly
                :value="getEntity[property.property]"
              >
              </v-textarea>
            </v-list-item-content>
          </v-list-item>
        </v-list>
        <EntityEventList
          :entityType="entityType"
          :entityId="getEntity.id"
          :watchEvent="watchEvent"
        />
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";
import DocumentNode from "graphql";

import { siComponentRegistry } from "@/registry";
import { SiComponent } from "@/registry/siComponent";
import { Query } from "@/graphql-types";

import EntityEventList from "@/components/EntityEventList.vue";

interface DataField {
  getEntity: any;
  siComponent: SiComponent;
  watchEvent: number;
}

export default Vue.extend({
  name: "EntityShow",
  props: {
    entityName: String,
    entityType: String,
    entityId: String,
  },
  data(): DataField {
    const siComponent = siComponentRegistry.lookup(this.entityType);
    const watchEvent = 0;
    return {
      getEntity: {},
      siComponent,
      watchEvent,
    };
  },
  methods: {
    runAction(entityId: string, mutation: DocumentNode): void {
      if (mutation) {
        this.$apollo.mutate({
          mutation,
          variables: {
            entityId,
          },
          update: (store, { data: { runAction } }) => {
            this.watchEvent++;
            //this.$emit("refresh:entity-event-list", runAction);
            //const listEntityEvents = this.siComponent.listEntityEvents;
            //// Read the data from our cache for this query.
            //const data = store.readQuery({ query: listEntityEvents });
            //// Add our tag from the mutation to the end
            //data.tags.push(addTag);
            //// Write our data back to the cache.
            //store.writeQuery({ query: TAGS_QUERY, data });
          },
        });
      }
    },
  },
  apollo: {
    getEntity: {
      query(): any {
        let siComponent = siComponentRegistry.lookup(this.entityType);
        return siComponent.getEntity;
      },
      update(data: any): any {
        let resultString = siComponentRegistry
          .lookup(this.entityType)
          .getEntityResultString();

        if (data[resultString] && data[resultString]["entity"]) {
          return data[resultString]["entity"];
        } else {
          return {};
        }
      },
      variables(): any {
        return {
          entityId: this.entityId,
        };
      },
    },
  },
  components: { EntityEventList },
});
</script>
