const path = require("path");
const nodeExternals = require("webpack-node-externals");
const NodemonPlugin = require("nodemon-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const WebpackBar = require("webpackbar");

module.exports = {
  mode: "development",
  devtool: "inline-source-map",
  externals: [nodeExternals()],
  // Change to your "entry-point".
  entry: {
    main: "./src/index",
    binary: "./src/bin/si-generate",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].bundle.js",
  },
  resolve: {
    extensions: [".ts", ".js", ".json"],
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  module: {
    rules: [
      {
        // Include ts, tsx, js, and jsx files.
        test: /\.(ts|js)x?$/,
        exclude: /node_modules/,
        include: [path.resolve("."), path.resolve("../si-registry")],
        loader: "babel-loader",
      },
    ],
  },
  plugins: [
    new WebpackBar(),
    new CleanWebpackPlugin(),
    new NodemonPlugin({
      script: "./dist/binary.bundle.js",
      watch: ["./dist", "./src/codegen/rust"],
      nodeArgs: ["--enable-source-maps"],
      ext: "js,ejs",
    }),
  ],
  target: "node",
};
