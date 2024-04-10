import path from 'path'
import { defineConfig } from 'cypress'
import vitePreprocessor from 'cypress-vite'

export default defineConfig({
  e2e: {
    setupNodeEvents(on, config) {
      on('file:preprocessor', 
        vitePreprocessor(
          path.resolve('./vite.config.ts'),
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
    baseUrl: 'http://localhost:8080/',
    chromeWebSecurity: false,
    viewportHeight: 1000,
    viewportWidth: 1500,
  },
  projectId: "k8tgfj",
  video: true,
})

