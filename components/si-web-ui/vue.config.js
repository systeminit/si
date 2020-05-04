module.exports = {
  transpileDependencies: ["vuetify"],
  configureWebpack: {
    resolve: {
      symlinks: false,
    },
  },
  pluginOptions: {
    apollo: {
      lintGQL: false,
    },
  },
};
