const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");

module.exports = {
  configureWebpack: {
    plugins: [
      new MonacoWebpackPlugin({
        languages: ["yaml"],
      }),
    ],
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
