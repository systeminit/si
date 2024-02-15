import path from 'path'
import { defineConfig } from 'cypress'
import vitePreprocessor from 'cypress-vite'



export default defineConfig({
  e2e: {
    setupNodeEvents(on, config) {
      on('file:preprocessor', vitePreprocessor(path.resolve('./vite.config.ts'),
      ))
    },
    baseUrl: 'https://auth.systeminit.com',
    chromeWebSecurity: false,
    viewportHeight: 1000,
    viewportWidth: 1500,
  }
})

