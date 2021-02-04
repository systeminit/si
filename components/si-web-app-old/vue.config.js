//const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");

module.exports = {
  configureWebpack: {
    resolve: {
      symlinks: false,
    },
    devtool: "source-map",
  },
};
