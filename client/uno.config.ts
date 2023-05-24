/* eslint-disable import/no-extraneous-dependencies */
import { defineConfig } from 'unocss/vite';
import {
  presetWind,
  presetAttributify,
  transformerAttributifyJsx,
  transformerVariantGroup,
  transformerDirectives,
} from 'unocss';

export default defineConfig({
  presets: [presetWind(), presetAttributify()],
  transformers: [transformerAttributifyJsx(), transformerVariantGroup(), transformerDirectives()],
});
