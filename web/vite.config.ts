import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tsConfig from 'vite-tsconfig-paths'

// https://vitejs.dev/config/
export default defineConfig({
	server: {
		port: 3000,
		hmr: {
			host: 'localhost',
			clientPort: 2069,
			protocol: 'wss',
		}
	},
	plugins: [react(), tsConfig()]
})
