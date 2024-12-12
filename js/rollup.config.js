import { nodeResolve } from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';

const isProduction = process.env.NODE_ENV === 'production';

export default {
  input: 'src/index.ts',
  output: {
    file: '../public/inferno.ext.js',
    format: 'iife',
    name: 'Inferno'
  },
  plugins: [
    typescript(),
    nodeResolve(),
    isProduction && (await import('@rollup/plugin-terser')).default()
  ]
};
