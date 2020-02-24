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
        <EntityEventList :entityType="entityType" :entityId="getEntity.id" />
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
    return {
      getEntity: {},
      siComponent,
    };
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
