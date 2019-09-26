const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const NodemonPlugin = require("nodemon-webpack-plugin");
const merge = require("webpack-merge");
const nodeExternals = require("webpack-node-externals");
const path = require("path");
const webpack = require("webpack");

const common = require("./webpack.common.js");

module.exports = merge.smart(common, {
  devtool: "inline-source-map",
  //entry: [
  //  'webpack-hot-middleware/client?path=/__webpack_hmr&timeout=20000',
  //  path.join(__dirname, 'src/main.ts'),
  //],
  externals: [nodeExternals()],
  //externals: [
  //  nodeExternals({
  //    whitelist: ['webpack/hot/poll?1000'],
  //  }),
  //],
  mode: "development",
  plugins: [
    new CleanWebpackPlugin(),
    new NodemonPlugin({
      script: "./dist/server.js",
      watch: "./dist/server.js",
    }),
  ],
  watch: true,
});
