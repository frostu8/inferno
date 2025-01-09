import { nodeResolve } from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import sass from 'rollup-plugin-sass';
import copy from 'rollup-plugin-copy';

const isProduction = process.env.NODE_ENV === 'production';
const outputPath = process.env.WEBPACK_OUT_DIR || './build';

export default {
  input: 'src/index.ts',
  output: {
    file: `${outputPath}/inferno.js`,
    format: 'iife',
    name: 'Inferno'
  },
  plugins: [
    sass({ output: `${outputPath}/inferno.css` }),
    copy({
      targets: [
        { src: 'assets/favicon.ico', dest: outputPath }
      ]
    }),
    typescript(),
    nodeResolve(),
    isProduction && (await import('@rollup/plugin-terser')).default()
  ]
};
