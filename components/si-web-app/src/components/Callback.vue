<template>
  <div class="spinner">
    <img src="../assets/loading.svg" alt="Loading" />
  </div>
</template>

<script>
export default {
  methods: {
    handleLoginEvent(data) {
      if (!data.error) {
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
