<template>
  <v-container class="justify-start d-flex" sytle="flex-direction: row;">
    <v-row no-gutters class="justify-start">
      <v-col cols="12">
        <v-card cols="12">
          <v-alert type="error" dismissible v-if="errorMessage">
            {{ errorMessage }}
          </v-alert>

          <v-card-title>
            Events
          </v-card-title>
          <v-card-text>
            <v-data-table
              :headers="headers"
              :items="listEntityEvents.items"
              :server-items-length="listEntityEvents.totalCount"
              :options.sync="options"
              :loading="loading"
              v-on:update:page="showMore"
              show-expand
              hide-default-footer
            >
              <template v-slot:expanded-item="data">
                <td :colspan="data.headers.length">
                  <v-list>
                    <v-list-item>
                      <v-list-item-content>
                        <v-list-item-title>
                          Output
                        </v-list-item-title>
                        <v-textarea
                          on-resize
                          outlined
                          full-width
                          flat
                          readonly
                          :value="data.item.outputLines.join('\n')"
                        />
                      </v-list-item-content>
                    </v-list-item>
                    <v-list-item>
                      <v-list-item-content>
                        <v-list-item-title>
                          Error
                        </v-list-item-title>
                        <v-textarea
                          on-resize
                          outlined
                          full-width
                          flat
                          readonly
                          :value="data.item.errorLines.join('\n')"
                        />
                      </v-list-item-content>
                    </v-list-item>
                  </v-list>
                </td>
              </template>
            </v-data-table>
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-card class="d-flex justify-center align-content-center" flat>
              <v-card flat class="align-self-center pa-2">
                {{ listEntityEvents.items.length }} /
                {{ listEntityEvents.totalCount }} items
              </v-card>
              <v-card flat class="align-self-center pa-2 flex-grow-0" cols="1">
                <v-select
                  small-chips
                  v-model="options.itemsPerPage"
                  :items="itemsPerPageOptions"
                  label="Items Per Page"
                >
                </v-select>
              </v-card>
              <v-card flat class="align-self-center pa-2">
                <v-btn @click="showMore" :disabled="showMoreDisabled">
                  Load More
                </v-btn>
              </v-card>
            </v-card>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="js">
import Vue from "vue";

import { auth } from "@/auth";
import { siComponentRegistry } from "@/registry";

export default Vue.extend({
  name: "EntityEventList",
  props: {
    entityType: String,
    entityId: String,
    watchEvent: Number,
  },
  data() {
    let siComponent = siComponentRegistry.lookup(this.entityType);
    let headers = siComponent.listEntityEventHeaders;

    return {
      errorMessage: null,
      itemsPerPageOptions: [10, 20, 30, 40, 50, 100],
      options: {
        itemsPerPage: 10,
        sortBy: ["updatedTime"],
        sortDesc: [true],
        page: 1,
      },
      siComponent,
      headers,
      listEntityEvents: {
        items: [],
        totalCount: 0,
        nextPageToken: "",
      },
      loading: true,
      nextPageToken: "",
      showMoreDisabled: true,
    };
  },
  watch: {
    watchEvent(_oldNumber, _newNumber) {
      console.log("I got this");
      this.$apollo.queries.listEntityEvents.refetch();
    }
  },
  methods: {
    showMore() {
      this.loading = true;
      // Fetch more data and transform the original result
      this.$apollo.queries.listEntityEvents.fetchMore({
        // New variables
        variables: {
          pageToken: this.nextPageToken,
        },
        // Transform the previous result with new data
        updateQuery: (previousResult, { fetchMoreResult }) => {
          this.loading = false;
          let siComponent = siComponentRegistry.lookup(this.entityType);
          let resultString = siComponent.listEntityEventsResultString();
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
    listEntityEvents: {
      query() {
        let siComponent = siComponentRegistry.lookup(this.entityType);
        return siComponent.listEntityEvents;
      },
      variables() {
        let orderByDirection;
        if (this.options["sortDesc"][0]) {
          orderByDirection = "DESC";
        } else {
          orderByDirection = "ASC";
        }
        return {
          scopeByTenantId: this.entityId,
          pageSize: this.options["itemsPerPage"],
          orderBy: this.options["sortBy"][0],
          orderByDirection,
        };
      },
      update(data) {
        this.loading = false;
        let list;
        let siComponent = siComponentRegistry.lookup(this.entityType);
        let resultString = siComponent.listEntityEventsResultString();

        if (data[resultString]) {
          list = data[resultString];
        } else {
          list = this.listEntityEvents;
        }

        this.nextPageToken = list.nextPageToken || "";
        if (this.nextPageToken == "") {
          this.showMoreDisabled = true;
        } else {
          this.showMoreDisabled = false;
        }
        return list;
      },
      error(error, vm, key, type, options) {
        this.errorMessage = error.message;
      },
    },
  },
});
</script>
