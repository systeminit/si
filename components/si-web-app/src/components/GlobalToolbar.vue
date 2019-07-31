<template>
  <div>
    <v-app-bar fixed app color="primary" dark>
      <v-app-bar-nav-icon @click.stop="drawer = !drawer"></v-app-bar-nav-icon>
      <v-toolbar-title>System Initiative</v-toolbar-title>
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
              <img :src="profile.picture" />
            </v-avatar>
            <!-- <v-icon>person</v-icon> -->
          </v-btn>
        </template>

        <v-card>
          <v-list>
            <v-list-item>
              <v-list-item-content>
                <v-list-item-title>{{ profile.name }}</v-list-item-title>
                <v-list-item-subtitle>{{ profile.email }}</v-list-item-subtitle>
              </v-list-item-content>
            </v-list-item>
          </v-list>

          <v-divider></v-divider>

          <v-list>
            <v-list-item>
              <v-list-item-action>
                <v-switch v-model="message" color="purple"></v-switch>
              </v-list-item-action>
              <v-list-item-title>Enable messages</v-list-item-title>
            </v-list-item>

            <v-list-item>
              <v-list-item-action>
                <v-switch v-model="hints" color="purple"></v-switch>
              </v-list-item-action>
              <v-list-item-title>Enable hints</v-list-item-title>
            </v-list-item>
          </v-list>

          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn text @click="$auth.logOut()">Logout</v-btn>
            <v-btn text @click="menu = false">Cancel</v-btn>
          </v-card-actions>
        </v-card>
      </v-menu>
    </v-app-bar>
    <v-navigation-drawer v-model="drawer" app>
      <v-list dense>
        <v-list-item :to="{ name: 'home' }" exact append>
          <v-list-item-action>
            <v-icon>home</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Home</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <v-list-item :to="{ name: 'workspaces' }" exact append>
          <v-list-item-action>
            <v-icon>group_work</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Workspaces</v-list-item-title>
          </v-list-item-content>
        </v-list-item>

        <v-list-item :to="{ name: 'workspaces' }" exact append>
          <v-list-item-action>
            <v-icon>cloud</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Integrations</v-list-item-title>
          </v-list-item-content>
        </v-list-item>

        <v-list-item :to="{ name: 'about' }" exact append>
          <v-list-item-action>
            <v-icon>contact_mail</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Contact</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
export default Vue.extend({
  name: "GlobalToolbar",
  methods: {
    handleLoginEvent(data: any) {
      this.profile = data.profile;
    },
  },
  data() {
    return {
      profile: this.$auth.profile,
      drawer: false,
      menu: false,
      message: false,
      hints: true,
    };
  },
});
</script>
