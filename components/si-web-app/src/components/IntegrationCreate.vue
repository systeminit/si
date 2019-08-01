<template>
  <div>
    <v-card>
      <v-img height="100" contain :src="require(`../assets/${image}`)"></v-img>
      <v-card-title>
        <div>{{ description }}</div>
      </v-card-title>
      <v-card-actions>
        <v-spacer />
        <v-btn icon @click="show = !show">
          <v-icon>{{
            show ? "keyboard_arrow_down" : "keyboard_arrow_up"
          }}</v-icon>
        </v-btn>
      </v-card-actions>
      <v-expand-transition>
        <div v-show="show">
          <v-card-text>
            Add a new {{ name }} integration!
            <v-text-field
              required
              outlined
              v-model="newName"
              label="Integration Name"
            ></v-text-field>
            <v-text-field
              required
              outlined
              v-model="newDescription"
              label="Integration Description"
            ></v-text-field>
            <v-text-field
              v-for="field in getOptions()"
              :key="field.field_id"
              :label="field.field_name"
              v-model="newOptions[field.field_id]"
              required
              outlined
            ></v-text-field>
          </v-card-text>
          <v-card-actions>
            <v-spacer />
            <v-btn class="primary" @click="createIntegration()"
              >Add Integration</v-btn
            >
          </v-card-actions>
        </div>
      </v-expand-transition>
    </v-card>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import sortBy from "lodash/sortBy";

import getAllIntegrations from "@/graphql/queries/getAllIntegrations.graphql";
import getIntegrationInstances from "@/graphql/queries/getIntegrationInstances.graphql";
import createIntegrationInstance from "@/graphql/mutation/createIntegrationInstance.graphql";

export default Vue.extend({
  name: "IntegrationCreate",
  methods: {
    createIntegration() {
      this.$apollo.mutate({
        mutation: createIntegrationInstance,
        variables: {
          name: this.newName,
          description: this.newDescription,
          options: JSON.stringify(this.newOptions),
          integrationId: this.integrationId,
        },
        update: (store, createData) => {
          const integrationInstance =
            createData.data.createIntegrationInstance.integrationInstance;
          const data: any = store.readQuery({
            query: getIntegrationInstances,
          });
          data.getIntegrationInstances.push(integrationInstance);
          let nw = sortBy(data.getIntegrationInstances, ["name", "id"]);
          data.getIntegrationInstances = nw;
          store.writeQuery({
            query: getIntegrationInstances,
            data,
          });
          this.newName = "";
          this.newDescription = "";
          this.newOptions = {};
          this.$router.push({
            name: "integration",
            params: { id: integrationInstance.id },
          });
        },
      });
    },
    getOptions(): Array<Object> {
      let optionData = JSON.parse(this.options);
      console.log(optionData);
      return optionData.fields;
    },
  },
  data() {
    return {
      show: false,
      newName: "",
      newDescription: "",
      newOptions: {},
      errorMessage: "",
    };
  },
  props: {
    integrationId: String,
    name: String,
    description: String,
    image: String,
    options: String,
  },
});
</script>
