// @ts-check
///<reference path="../global.d.ts"/>

Cypress._.times(import.meta.env.VITE_SI_CYPRESS_MULTIPLIER ? import.meta.env.VITE_SI_CYPRESS_MULTIPLIER : 1, () => {
  describe('component', () => {
    beforeEach(function () {
      cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
    });

    it('create', () => {
      cy.visit('/')
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', 'Change Set 1');
      
      cy.get('#vorm-input-3').clear().type(import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

      cy.contains('Create change set', { timeout: 30000 }).click();

      // Give time to redirect onto the new changeset
      cy.url().should('not.include', 'head', { timeout: 10000 });

      // Find the AWS Credential
      cy.get('div[class="tree-node"]', { timeout: 30000 }).contains('AWS Credential').as('awsCred')

      // Find the canvas to get a location to drag to
      cy.get('canvas').first().as('konvaStage');

      // drag to the canvas
      cy.dragTo('@awsCred', '@konvaStage');

      //check to make sure a component has been added to the outliner
      cy.get('[class="component-outline-node"]', { timeout: 30000 }).contains('AWS Credential', { timeout: 30000 }).should('be.visible');

      // Click the button to destroy changeset
      cy.get('nav.navbar button.vbutton.--variant-ghost.--size-sm.--tone-action')
      .eq(1) // Selects the second button (index starts from 0 for create changeset button)
      .click();

      // Wait for the delete panel to appear
      cy.wait(1000);

      // Then click the agree button in the UI
      cy.get('button.vbutton.--variant-solid.--size-md.--tone-destructive')
      .click();

    })
  })

});