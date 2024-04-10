// @ts-check
///<reference path="../../global.d.ts"/>

const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe('component', () => {
    beforeEach(function () {
      //cy.setupVariables();
      cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);
      cy.log("fuck yeah i logged in!");
    });

    it('create', () => {
      cy.visit(AUTH_API_URL + '/workspaces/' + SI_WORKSPACE_ID + '/go');
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', 'Change Set 1');
      
      cy.get('#vorm-input-3').clear().type(UUID);

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', UUID);

      cy.contains('Create change set', { timeout: 30000 }).click();

      // Give time to redirect onto the new changeset
      cy.url().should('not.include', 'head', { timeout: 10000 });

      // Create a region component
      cy.createComponent("region");

      // // Find the AWS Region
      // cy.get('div[class="tree-node"]', { timeout: 30000 }).contains('Region').as('region');

      // // Find the canvas to get a location to drag to
      // cy.get('canvas').first().as('konvaStage');

      // // drag to the canvas
      // cy.dragTo('@region', '@konvaStage');

      // cy.wait(5000);

      // //check to make sure a component has been added to the outliner
      // cy.get('[class="component-outline-node"]', { timeout: 30000 }).contains('Region', { timeout: 30000 }).should('be.visible');

      // Click the button to destroy changeset
      cy.get('nav.navbar button.vbutton.--variant-ghost.--size-sm.--tone-action')
      .eq(1) // Selects the second button (index starts from 0 for create changeset button)
      .click();

      // Wait for the delete panel to appear
      cy.wait(1000);

      // Then click the agree button in the UI
      cy.get('button.vbutton.--variant-solid.--size-md.--tone-destructive')
      .click();

    });
  });
});
