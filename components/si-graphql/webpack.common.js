const path = require('path');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');

module.exports = {
  module: {
    rules: [
      {
        exclude: [path.resolve(__dirname, 'node_modules')],
        test: /\.ts$/,
        use: 'ts-loader'
      },
      {
        test: /\.(graphql|gql)$/,
        exclude: [path.resolve(__dirname, 'node_modules')],
        loader: 'graphql-tag/loader',
      }
    ]
  },
  output: {
    filename: 'server.js',
    path: path.resolve(__dirname, 'dist')
  },
  resolve: {
    plugins: [ new TsconfigPathsPlugin({})],
    extensions: ['.ts', '.tsx', '.js', '.graphql', '.gql' ]
  },
  target: 'node'
};
