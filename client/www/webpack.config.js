const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
  devServer: {
    publicPath: '/pong/web/',
    openPage: 'pong/web/',
    open: true,
    proxy: {
      '/pong/api': {
        target: 'http://localhost:4000',
        pathRewrite: { '^/pong/api': '' }
      }
    }
  },
};
