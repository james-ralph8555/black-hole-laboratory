const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "development",
  entry: {
    index: "./bootstrap.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  devServer: {
    static: {
        directory: dist,
    }
  },
  plugins: [
    new CopyPlugin({
        patterns: [
            path.resolve(__dirname, "index.html")
        ]
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".."),
      extraArgs: "--target web -- -p renderer",
      outDir: path.resolve(__dirname, "pkg"),
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
};
