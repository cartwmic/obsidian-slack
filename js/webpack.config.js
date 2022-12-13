const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");
const rustObsidianSlack = path.resolve("../rust/obsidian-slack")

module.exports = {
  mode: "production",
  entry: {
    index: "./bootstrap.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: rustObsidianSlack,
    }),
  ]
};