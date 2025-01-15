const path = require('path');

/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  context: __dirname,
  entry: {
    main: "./src/index.jsx"
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
  },
  resolve: {
    extensions: ['.js', '.jsx']
  },
  module: {
    rules: [
      {
        test: /\.jsx?$/,
        include: path.resolve(__dirname, 'src'),
        use: {
          loader: "builtin:swc-loader",
          options: {
            jsc: {
              parser: {
                syntax: "ecmascript",
                jsx: true
              },
              transform: {
                react: {
                  runtime: "automatic"
                }
              },
              experimental: {
                plugins: [
                  [path.resolve(__dirname, '../swc_plugin_jsx_css_modules.wasm'), {
                    prefer: 'global',
                    styleFileReg: ['\.module\.(css|scss|sass|less)$']
                  }]
                ]
              }
            }
          }
        }
      },
      {
        test: /\.module\.css$/,
        type: "css/module"
      }
    ]
  },
  builtins: {
    html: [
      {
        template: "./src/index.html"
      }
    ]
  }
};
