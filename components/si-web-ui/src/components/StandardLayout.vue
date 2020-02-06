<template>
  <div>
    <v-navigation-drawer app clipped fixed expand-on-hover floating>
      <v-list dense>
        <v-list-group prepend-icon="mdi-view-dashboard" class="white--text">
          <template v-slot:activator>
            <v-list-item-title>Workspaces</v-list-item-title>
          </template>
          <v-list-item>
            <v-list-item
              link
              :to="{
                name: 'workspace',
                params: {
                  organizationId: organization.id,
                  workspaceId: workspace.id,
                },
              }"
            >
              <v-list-item-action>
                <v-icon>mdi-alpha-d</v-icon>
              </v-list-item-action>
              <v-list-item-content>
                <v-list-item-title>Default</v-list-item-title>
              </v-list-item-content>
            </v-list-item>
          </v-list-item>
        </v-list-group>
        <v-list-group prepend-icon="mdi-dns-outline" class="white--text">
          <template v-slot:activator>
            <v-list-item-title>Entities</v-list-item-title>
          </template>
          <v-list-item
            link
            :to="{
              name: 'workspaceCreateEntity',
              params: {
                organizationId: organization.id,
                workspaceId: workspace.id,
                entityType: 'sshKey',
              },
            }"
          >
            <v-list-item-action>
              <v-icon>mdi-key</v-icon>
            </v-list-item-action>
            <v-list-item-content>
              SSH Key
            </v-list-item-content>
          </v-list-item>
        </v-list-group>

        <!--
        <v-list-item
          link
          :to="{
            name: 'workspace',
            params: {
              organizationId: organization.id,
              workspaceId: workspace.id,
            },
          }"
        >
          <v-list-item-action>
            <v-icon>mdi-view-dashboard</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Default Workspace</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
-->
        <v-list-item link :to="{ name: 'about' }">
          <v-list-item-action>
            <v-icon>mdi-settings</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Settings</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-app-bar app clipped-left>
      <!-- <v-app-bar-nav-icon @click.stop="drawer = !drawer" /> -->
      <v-toolbar-title
        >IRA OS / Org: {{ organization.name }} / Workspace:
        {{ workspace.name }}</v-toolbar-title
      >
      <v-spacer></v-spacer>
      <v-menu
        v-model="menu"
        :close-on-content-click="false"
        :nudge-width="200"
        offset-y
      >
        <template v-slot:activator="{ on }">
          <v-btn icon v-on="on">
            <v-avatar>
              <img v-if="profile.picture" :src="profile.picture" />
              <v-icon v-else>mdi-account</v-icon>
            </v-avatar>
            <!-- <v-icon>mdi-person</v-icon> -->
          </v-btn>
        </template>

        <v-card>
          <v-list>
            <v-list-item>
              <v-list-item-content>
                <v-list-item-title>
                  {{ profile.displayName }} @
                  {{ profile.billingAccount.displayName }}
                </v-list-item-title>
                <v-list-item-subtitle>{{ profile.email }}</v-list-item-subtitle>
              </v-list-item-content>
            </v-list-item>
          </v-list>

          <v-divider></v-divider>

          <v-list>
            <v-list-item>
              <v-list-item-action>
                <v-switch v-model="message" color="secondary"></v-switch>
              </v-list-item-action>
              <v-list-item-title>Enable messages</v-list-item-title>
            </v-list-item>

            <v-list-item>
              <v-list-item-action>
                <v-switch v-model="hints" color="secondary"></v-switch>
              </v-list-item-action>
              <v-list-item-title>Enable hints</v-list-item-title>
            </v-list-item>
          </v-list>

          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn text @click="logOut()">Logout</v-btn>
            <v-btn text @click="menu = false">Cancel</v-btn>
          </v-card-actions>
        </v-card>
      </v-menu>
    </v-app-bar>

    <v-content>
      <slot></slot>
    </v-content>

    <v-footer app>
      <span>&copy; 2020 - The System Initiative, Inc.</span>
    </v-footer>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import { auth } from "@/auth";

export default Vue.extend({
  name: "StandardLayout",
  data: () => {
    const profile = auth.getProfile();
    const organization = (profile &&
      profile.billingAccount &&
      profile.billingAccount.organizations &&
      profile.billingAccount.organizations.items &&
      profile.billingAccount.organizations.items[0]) || { name: "busted" };
    const workspace = (organization &&
      organization.workspaces &&
      organization.workspaces.items &&
      organization.workspaces.items[0]) || { name: "busted" };
    return {
      drawer: false,
      message: false,
      hints: false,
      menu: false,
      profile,
      organization,
      workspace,
    };
  },
  methods: {
    async logOut() {
      await auth.logout();
      this.$router.push({ name: "signin" });
    },
  },
  created() {
    this.$vuetify.theme.dark = true;
  },
});
</script>
