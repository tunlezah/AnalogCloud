import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  // Pull VITE_* and ANALOG_CLOUD_* keys from process.env / .env. The
  // repo-local .env at the workspace root is written by scripts/install.sh
  // and contains the user's chosen ports; CI / hand-rolled setups can
  // pass the same variables via the shell.
  const env = { ...process.env, ...loadEnv(mode, process.cwd(), '') };

  const frontendPort = Number(env.VITE_FRONTEND_PORT ?? env.ANALOG_CLOUD_FRONTEND_PORT ?? 5173);
  const backendUrl =
    env.VITE_BACKEND_URL ??
    (env.ANALOG_CLOUD_BIND ? `http://${env.ANALOG_CLOUD_BIND}` : 'http://127.0.0.1:7777');

  return {
    plugins: [sveltekit()],
    server: {
      port: frontendPort,
      strictPort: true,
      proxy: {
        '/api': { target: backendUrl, changeOrigin: false },
        '/events': { target: backendUrl, changeOrigin: false, ws: true },
        '/ws': { target: backendUrl, changeOrigin: false, ws: true }
      }
    }
  };
});
