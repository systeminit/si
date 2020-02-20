<template>
  <v-container class="justify-start d-flex" sytle="flex-direction: row;">
    <v-row no-gutters class="justify-start">
      <v-col cols="12">
        <v-card cols="12">
          <v-card-title>
            {{ siComponent.name }}
            <v-spacer></v-spacer>
            <v-btn
              :to="{
                name: 'workspaceCreateEntity',
                params: {
                  organizationId: organizationId,
                  workspaceId: workspaceId,
                  entityType: siComponent.typeName,
                },
              }"
            >
              <v-icon>mdi-plus</v-icon>
            </v-btn>
          </v-card-title>
          <v-card-text>
            <v-data-table
              :headers="headers"
              :items="listEntities.items"
              :server-items-length="listEntities.totalCount"
              :options.sync="options"
              :loading="loading"
              v-on:update:page="showMore"
              hide-default-footer
            >
              <template v-slot:item.name="{ item }">
                <router-link
                  :to="{
                    name: 'workspaceShowEntity',
                    params: {
                      organizationId: item.organizationId,
                      workspaceId: item.workspaceId,
                      entityId: item.id,
                      entityType: entityType,
                    },
                  }"
                  >{{ item.name }}</router-link
                >
              </template>
            </v-data-table>
          </v-card-text>
          <v-card-actions>
            <v-spacer> </v-spacer>
            Total Count: {{ listEntities.totalCount }} Items Per Page:
            {{ options.itemsPerPage }}
            <v-btn @click="showMore" :disabled="showMoreDisabled">
              Load More
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";

import { auth } from "@/auth";
import { siComponentRegistry } from "@/registry";
import { SiComponent } from "@/registry/siComponent";
import { Query, DataOrderByDirection } from "@/graphql-types";

interface EntityListData {
  options: {
    itemsPerPage: number;
    sortBy: string[];
    sortDesc: boolean[];
    page: number;
  };
  siComponent: SiComponent;
  headers: { text: string; value: string }[];
  listEntities: {};
  loading: boolean;
  nextPageToken: string;
  showMoreDisabled: boolean;
}

export default Vue.extend({
  name: "EntityList",
  props: {
    entityType: String,
    organizationId: String,
    workspaceId: String,
  },
  data(): EntityListData {
    let headers;
    let siComponent = siComponentRegistry.lookup(this.entityType);
    return {
      options: {
        itemsPerPage: 10,
        sortBy: ["displayName"],
        sortDesc: [false],
        page: 1,
      },
      siComponent,
      headers: siComponent.listHeaders,
      listEntities: {
        items: [],
        totalCount: 0,
        nextPageToken: "",
      },
      loading: true,
      nextPageToken: "",
      showMoreDisabled: true,
    };
  },
  methods: {
    showMore(): void {
      this.loading = true;
      // Fetch more data and transform the original result
      this.$apollo.queries.listEntities.fetchMore({
        // New variables
        variables: {
          pageToken: this.nextPageToken,
        },
        // Transform the previous result with new data
        updateQuery: (previousResult, { fetchMoreResult }) => {
          this.loading = false;
          let siComponent = siComponentRegistry.lookup(this.entityType);
          let resultString = siComponent.listEntitiesResultString();
          let newItems = fetchMoreResult[resultString].items;
          let nextPageToken = fetchMoreResult[resultString].nextPageToken;
          let nextTotalCount = fetchMoreResult[resultString].totalCount;
          let previousResultList = previousResult[resultString];

          this.nextPageToken = nextPageToken;

          if (this.nextPageToken == "") {
            this.showMoreDisabled = true;
          } else {
            this.showMoreDisabled = false;
          }

          return {
            [resultString]: {
              __typename: previousResultList.__typename,
              // Merging the tag list
              items: [...previousResultList.items, ...newItems],
              totalCount: nextTotalCount,
              nextPageToken,
            },
          };
        },
      });
    },
  },
  apollo: {
    listEntities: {
      query(): any {
        let siComponent = siComponentRegistry.lookup(this.entityType);
        return siComponent.listEntities;
      },
      update(data: Query): any {
        this.loading = false;
        let listEntities;
        let siComponent = siComponentRegistry.lookup(this.entityType);
        let resultString = siComponent.listEntitiesResultString();

        // @ts-ignore - we know, you can't index it. but you can.
        if (data[resultString]) {
          // @ts-ignore same same
          listEntities = data[resultString];
        } else {
          // We got bullshit data, so.. just use the old data? <shrub>
          listEntities = this.listEntities;
        }
        this.nextPageToken = listEntities.nextPageToken || "";
        if (this.nextPageToken == "") {
          this.showMoreDisabled = true;
        } else {
          this.showMoreDisabled = false;
        }
        return listEntities;
      },
      variables(): any {
        let orderByDirection: DataOrderByDirection;
        if (this.options["sortDesc"][0]) {
          orderByDirection = DataOrderByDirection.Desc;
        } else {
          orderByDirection = DataOrderByDirection.Asc;
        }
        return {
          pageSize: this.options["itemsPerPage"],
          orderBy: this.options["sortBy"][0],
          orderByDirection,
        };
      },
    },
  },
});
</script>
