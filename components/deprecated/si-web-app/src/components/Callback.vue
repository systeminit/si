<template>
  <div class="spinner">
    <img src="../assets/loading.svg" alt="Loading" />
  </div>
</template>

<script>
import gql from "graphql-tag";
import createUser from "@/graphql/mutation/createUser.graphql";

export default {
  methods: {
    async handleLoginEvent(data) {
      if (!data.error) {
        let r = await this.$apollo.mutate({
          mutation: createUser,
          variables: {
            email: data.profile.email,
            name: data.profile.name,
          },
        });
        this.$router.push(data.state.target || "/");
      }
    },
  },
  async created() {
    try {
      await this.$auth.handleAuthentication();
    } catch (e) {
      this.$router.push("/");
      console.error(e);
    }
  },
};
</script>

<style scoped>
.spinner {
  position: absolute;
  display: flex;
  justify-content: center;
  height: 100vh;
  width: 100vw;
  background-color: white;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
}
</style>
