const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = (env, argv) => {
  const isProduction = argv.mode === 'production';
  
  return {
  mode: isProduction ? "production" : "development",
  entry: {
    index: "./bootstrap.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  devServer: {
    host: "0.0.0.0",
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
      crateDirectory: path.resolve(__dirname, "../renderer"),
      extraArgs: `--target web${isProduction ? ' --release' : ' --profile dev-release'}`,
      outDir: path.resolve(__dirname, "pkg"),
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
  };
};
