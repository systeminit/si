const path = require("path");
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

module.exports = {
  module: {
    rules: [
      {
        exclude: [path.resolve(__dirname, "node_modules")],
        test: /\.ts$/,
        use: "ts-loader",
      },
      {
        test: /\.(graphql|gql)$/,
        exclude: [path.resolve(__dirname, "node_modules")],
        loader: "graphql-tag/loader",
      },
    ],
  },
  entry: {
    server: path.join(__dirname, "src/main.ts"),
    migrate: path.join(__dirname, "src/migratedata.ts"),
  },
  output: {
    filename: "[name].js",
    path: path.resolve(__dirname, "dist"),
  },
  resolve: {
    plugins: [new TsconfigPathsPlugin({})],
    extensions: [".ts", ".tsx", ".js"],
  },
  target: "node",
};
