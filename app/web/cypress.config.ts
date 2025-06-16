import path from 'path'
import { defineConfig } from 'cypress'
import vitePreprocessor from 'cypress-vite'

export default defineConfig({
  e2e: {
    setupNodeEvents(on, config) {
      on('file:preprocessor',
        vitePreprocessor(
          path.resolve('./vite.cypress.ts'),
        )
      ),
        on('task', {
          log(message) {
            console.log(message)
            return null
          }
        })
    },

    // Hotfix, needs amended
    baseUrl: process.env.VITE_HOST_URL ? process.env.VITE_HOST_URL : 'http://127.0.0.1:8080',
    chromeWebSecurity: false,
    viewportHeight: 1000,
    viewportWidth: 1500,
    retries: process.env.VITE_SI_CYPRESS_MULTIPLIER ? Number(process.env.VITE_SI_CYPRESS_MULTIPLIER) : 1,
    screenshotsFolder: 'cypress/screenshots',
    videosFolder: 'cypress/videos',
  },
  projectId: "k8tgfj",
  video: true,
})

