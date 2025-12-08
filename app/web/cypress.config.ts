import path from 'path'
import { defineConfig } from 'cypress'
import vitePreprocessor from 'cypress-vite'

const FLAKY_EXIT_CODE = 53

export default defineConfig({
  e2e: {
    injectDocumentDomain: true,
    specPattern: "cypress/**/*.cy.{js,jsx,ts,tsx}",
    setupNodeEvents(on, config) {
      on('file:preprocessor',
        vitePreprocessor(
          path.resolve('./vite.cypress.ts'),
        )
      );
        
      on('task', {
        log(message) {
          console.log(message)
          return null
        },
        flakyFailure() {
          console.log('Flaky failure detected - exiting with code', FLAKY_EXIT_CODE)
          // Use setTimeout to allow Cypress to receive the return value before exiting
          setTimeout(() => {
            process.exit(FLAKY_EXIT_CODE)
          }, 100)
          return null
        }
      });

      on('after:run', (results: any) => {
        const hasAuth0Failures = results.runs?.some((run: any) => 
          run.tests?.some((test: any) => 
            test.displayError?.includes('Auth0') || 
            test.title?.includes('auth0') ||
            test.err?.message?.includes('Auth0')
          )
        );
        
        if (hasAuth0Failures) {
          console.log('Detected Auth0-related test failures - exiting with code', FLAKY_EXIT_CODE)
          process.exit(FLAKY_EXIT_CODE)
        }
      });
    },

    // Hotfix, needs amended
    baseUrl: process.env.VITE_HOST_URL ? process.env.VITE_HOST_URL : 'http://127.0.0.1:8080',
    chromeWebSecurity: false,
    viewportHeight: 1000,
    viewportWidth: 1500,
    retries: process.env.VITE_SI_CYPRESS_MULTIPLIER ? Number(process.env.VITE_SI_CYPRESS_MULTIPLIER) : 0,
  },
  projectId: "k8tgfj",
  video: true,
})

