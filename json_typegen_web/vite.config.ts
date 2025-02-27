import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { ViteRsw } from "vite-plugin-rsw";
import { viteStaticCopy } from "vite-plugin-static-copy";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    ViteRsw(),
    svelte(),
    viteStaticCopy({
      targets: [
        {
          src: "node_modules/shiki/dist/onig.wasm",
          dest: "shiki/dist",
        },
        {
          src: "node_modules/shiki/languages/*",
          dest: "shiki/languages",
        },
        {
          src: "node_modules/shiki/themes/*",
          dest: "shiki/themes",
        },
      ],
    }),
  ],
  server: {
    fs: {
      allow: [".."],
    },
  },
});
