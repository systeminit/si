<template>
  <div>
    <v-container>
      <v-row align="center" justify="center">
        <v-col xl="12">
          <v-card md8 flat>
            <v-toolbar flat>
              <v-toolbar-title>Integrations</v-toolbar-title>
              <template v-slot:extension>
                <v-tabs v-model="tabs" align-with-title>
                  <v-tab>
                    <v-icon>list</v-icon>
                  </v-tab>
                  <v-tab>
                    <v-icon>add</v-icon>
                  </v-tab>
                </v-tabs>
              </template>
            </v-toolbar>
            <v-divider />
            <v-tabs-items v-model="tabs">
              <v-tab-item key="0">
                <v-card-text v-if="getIntegrationInstances.length > 0">
                  <v-container>
                    <v-row align="center">
                      <v-col
                        v-for="integration in getIntegrationInstances"
                        :key="integration.id"
                        md="6"
                      >
                        <IntegrationCard
                          :integrationInstanceId="integration.id"
                          :name="integration.name"
                          :description="integration.description"
                          :integration="integration.integration"
                        />
                      </v-col>
                    </v-row>
                  </v-container>
                </v-card-text>
                <v-card-text v-else>
                  <v-container>
                    <v-row align="center" justify="center">
                      <v-col md="6">
                        <v-card>
                          <v-card-title>
                            <h2>You haven't enabled any integrations!</h2>
                          </v-card-title>
                          <v-card-text
                            >Integrations allow you to design and manage the
                            various components of the system.</v-card-text
                          >
                          <v-card-actions>
                            <v-spacer />
                            <v-btn @click="tabs = 1" color="primary" text
                              >Add a new integration</v-btn
                            >
                          </v-card-actions>
                        </v-card>
                      </v-col>
                    </v-row>
                  </v-container>
                </v-card-text>
              </v-tab-item>
              <v-tab-item key="1">
                <v-container>
                  <v-row align="center">
                    <v-col
                      md="6"
                      v-for="integration in getAllIntegrations"
                      :key="integration.id"
                    >
                      <IntegrationCreate
                        :integrationId="integration.id"
                        :name="integration.name"
                        :description="integration.description"
                        :options="integration.options"
                        :image="integration.image"
                      />
                    </v-col>
                  </v-row>
                </v-container>
              </v-tab-item>
            </v-tabs-items>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import IntegrationCard from "@/components/IntegrationCard.vue";
import IntegrationCreate from "@/components/IntegrationCreate.vue";
import getAllIntegrations from "@/graphql/queries/getAllIntegrations.graphql";
import getIntegrationInstances from "@/graphql/queries/getIntegrationInstances.graphql";

export default Vue.extend({
  name: "IntegrationTop",
  apollo: {
    getAllIntegrations: {
      query: getAllIntegrations,
    },
    getIntegrationInstances: {
      query: getIntegrationInstances,
    },
  },
  data() {
    return {
      getIntegrationsById: [],
      getAllIntegrations: [],
      getIntegrationInstances: [],
      tabs: 0,
    };
  },
  components: {
    IntegrationCard,
    IntegrationCreate,
  },
});
</script>
