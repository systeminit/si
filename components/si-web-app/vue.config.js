module.exports = {
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
