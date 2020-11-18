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
    devtool: "source-map",
  },
  pluginOptions: {
    apollo: {
      lintGQL: false,
    },
  },
};
