<template>
  <div>
    <v-navigation-drawer fixed v-model="drawer" app>
      <v-list dense>
        <v-list-tile :to="{ name: 'home' }" active-class append>
          <v-list-tile-action>
            <v-icon>home</v-icon>
          </v-list-tile-action>
          <v-list-tile-content>
            <v-list-tile-title>Home</v-list-tile-title>
          </v-list-tile-content>
        </v-list-tile>
        <v-list-tile :to="{ name: 'about' }" active-class append>
          <v-list-tile-action>
            <v-icon>contact_mail</v-icon>
          </v-list-tile-action>
          <v-list-tile-content>
            <v-list-tile-title>Contact</v-list-tile-title>
          </v-list-tile-content>
        </v-list-tile>
      </v-list>
    </v-navigation-drawer>

    <v-toolbar fixed app>
      <v-toolbar-side-icon @click.stop="drawer = !drawer"></v-toolbar-side-icon>
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
            <v-list-tile>
              <v-list-tile-content>
                <v-list-tile-title>{{ profile.name }}</v-list-tile-title>
                <v-list-tile-sub-title>{{
                  profile.email
                }}</v-list-tile-sub-title>
              </v-list-tile-content>
            </v-list-tile>
          </v-list>

          <v-divider></v-divider>

          <v-list>
            <v-list-tile>
              <v-list-tile-action>
                <v-switch v-model="message" color="purple"></v-switch>
              </v-list-tile-action>
              <v-list-tile-title>Enable messages</v-list-tile-title>
            </v-list-tile>

            <v-list-tile>
              <v-list-tile-action>
                <v-switch v-model="hints" color="purple"></v-switch>
              </v-list-tile-action>
              <v-list-tile-title>Enable hints</v-list-tile-title>
            </v-list-tile>
          </v-list>

          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn flat @click="$auth.logOut()">Logout</v-btn>
            <v-btn flat @click="menu = false">Cancel</v-btn>
          </v-card-actions>
        </v-card>
      </v-menu>
    </v-toolbar>
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
