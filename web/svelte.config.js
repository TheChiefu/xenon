/** @type {import("@sveltejs/vite-plugin-svelte").SvelteConfig} */
export default {
  onwarn: (warning, handler) => {
    if (warning.code === 'a11y_media_has_caption') return;
    handler(warning);
  },
}
