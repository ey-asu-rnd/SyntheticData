import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import type { UserConfig as ViteUserConfig } from 'vite';
import type { InlineConfig } from 'vitest/node';

interface UserConfig extends ViteUserConfig {
  test?: InlineConfig;
}

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    exclude: ['e2e/**'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/lib/test-utils/vitest-setup.ts'],
    // Ensure browser version of Svelte is used
    server: {
      deps: {
        inline: [/svelte/],
      },
    },
    // Use browser conditions to resolve Svelte correctly
    alias: [
      {
        find: /^svelte\/?/,
        replacement: (id: string) => id.replace('svelte', 'svelte'),
        customResolver: {
          resolveId(source) {
            return null; // Let Vite handle it with browser conditions
          },
        },
      },
    ],
  },
  resolve: {
    conditions: ['browser'],
  },
} as UserConfig);
