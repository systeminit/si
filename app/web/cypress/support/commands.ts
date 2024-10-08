// @ts-check
///<reference path="../global.d.ts"/>

// ***********************************************
// This example commands.js shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
//
//
// -- This is a parent command --
// Cypress.Commands.add('login', (email, password) => { ... })
//
//
// -- This is a child command --
// Cypress.Commands.add('drag', { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add('dismiss', { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite('visit', (originalFn, url, options) => { ... })

// Import commands for auth0 auth providers
import "./auth-provider-commands/auth0";

// Import commands for posthog
import "./posthog-commands/posthog";
import "./component-actions/create";

Cypress.Commands.add("getBySel", (selector, ...args) => {
  return cy.get(`[data-test=${selector}]`, ...args)
})
Cypress.Commands.add("getBySelLike", (selector, ...args) => {
  return cy.get(`[data-test*=${selector}]`, ...args)
})

// Define the custom Cypress command in your Cypress support/commands.ts file
// commands.ts
Cypress.Commands.add('dragTo', (sourceElement: string, targetElement: string, offsetX?: number, offsetY?: number) => {
  cy.get(sourceElement).then(($sourceElement) => {
    cy.get(targetElement).then(($targetElement) => {
      const targetOffset = $targetElement.offset();
      const targetWidth = $targetElement.width();
      const targetHeight = $targetElement.height();

      // Calculate the coordinates to move to the center of the target element
      let clientX = targetOffset.left + targetWidth / 2;
      let clientY = targetOffset.top + targetHeight / 2;

      if (offsetX) clientX += offsetX;
      if (offsetY) clientY += offsetY;

      // Trigger a drag and drop action
      const sourceOffset = $sourceElement.offset();
      const drag = cy.get(sourceElement)
        .trigger('mousemove', { clientX: sourceOffset?.left, clientY: sourceOffset?.top, force: true })
        .trigger('mousedown', { button: 0, force: true }) // Simulate mouse down event
        .wait(10)
        .trigger('mousemove', { clientX: clientX, clientY: clientY, force: true }) // Move the object to a new position
        .wait(10)
        .trigger('mouseup', { button: 0 }) // Simulate mouse up event
    })
  })
});

Cypress.Commands.add('appModelPageLoaded', () => {
  cy.get('#asset-palette-title', { timeout: 60000 });
})

Cypress.Commands.add('clickButtonByIdIfExists', (id: string) => {
  cy.get('body').then(($body) => {
    // Check if element exists
    if ($body.find(`#${id}`).length > 0) {
      // Element exists, perform click
      cy.log(`Trying to click the #${id} now`);
      // Sometimes the div is hidden behind another modal, the force allows it to be clicked anyway
      cy.get(`#${id}`).click( { force: true });
    } else {
      // Element does not exist, continue with other actions
      cy.log('Button not found, continuing with other actions');
    }
  });
});

//Cypress.Commands.add('setupVariables', () => {
//  const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
//  const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
//  const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
//  const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
//  const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
//  const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";
//
//  Cypress.env('SI_CYPRESS_MULTIPLIER', SI_CYPRESS_MULTIPLIER);
//  Cypress.env('AUTH0_USERNAME', AUTH0_USERNAME);
//  Cypress.env('AUTH0_PASSWORD', AUTH0_PASSWORD);
//  Cypress.env('AUTH_API_URL', AUTH_API_URL);
//  Cypress.env('SI_WORKSPACE_ID', SI_WORKSPACE_ID);
//  Cypress.env('UUID', UUID);
//});