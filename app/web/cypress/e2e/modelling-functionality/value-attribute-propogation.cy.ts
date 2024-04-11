// @ts-check
///<reference path="../global.d.ts"/>

const SI_CYPRESS_MULTIPLIER = Cypress.env('VITE_SI_CYPRESS_MULTIPLIER') || import.meta.env.VITE_SI_CYPRESS_MULTIPLIER || 1;
const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const AUTH_API_URL = Cypress.env('VITE_AUTH_API_URL') || import.meta.env.VITE_AUTH_API_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const SI_WORKSPACE_URL = Cypress.env('VITE_SI_WORKSPACE_URL') || import.meta.env.VITE_SI_WORKSPACE_URL;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

Cypress._.times(SI_CYPRESS_MULTIPLIER, () => {
  describe('component', () => {
    beforeEach(function () {
      cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);
    });

    it('value_propagation', () => {
      console.log(UUID);
      cy.log(UUID);

      // Go to the Synthetic Workspace
      cy.visit(SI_WORKSPACE_URL + '/w/' + SI_WORKSPACE_ID + '/head');
      cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);
      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', 'Change Set 1');
      
      cy.get('#vorm-input-3').clear().type(UUID);

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', UUID);

      cy.contains('Create change set', { timeout: 30000 }).click();

      // Give time to redirect onto the new changeset
      cy.url().should('not.include', 'head', { timeout: 10000 });

      // Find the AWS Credential
      cy.get('div[class="tree-node"]', { timeout: 30000 }).contains('Region').as('awsRegion');

      // Find the canvas to get a location to drag to
      cy.get('canvas').first().as('konvaStage');

      cy.intercept('POST', '/api/diagram/create_component').as('componentA');
      let componentIDA, componentIDB;

      // drag to the canvas
      cy.dragTo('@awsRegion', '@konvaStage');
      cy.wait('@componentA', {timeout: 60000}).then(async (interception) => {
        componentIDA = interception.response?.body.componentId;
      });

      cy.wait(1000);

      cy.get('div[class="tree-node"]', { timeout: 30000 }).contains('EC2 Instance').as('awsEC2');

      cy.intercept('POST', '/api/diagram/create_component').as('componentB');
      cy.dragTo('@awsEC2', '@konvaStage', 0, 75);

      cy.wait('@componentB', {timeout: 60000}).then(async (interception) => {
        componentIDB = interception.response?.body.componentId;
      });

      cy.wait(1000);

      cy.url().then(currentUrl => {
        // Construct a new URL with desired query parameters for selecting 
        // the attribute panel for a known component
        let newUrl = new URL(currentUrl);
        newUrl.searchParams.set('s', 'c_'+componentIDA);
        newUrl.searchParams.set('t', 'attributes');
      
        // Visit the new URL
        console.log(newUrl.href);
        cy.visit(newUrl.href);
      });

      // Give the page a few seconds to load
      cy.wait(1000);

      cy.intercept('POST', '/api/component/update_property_editor_value').as('updatePropertyEditorValue');

      // Find the attribute for the Integer Input
      cy.get('.attributes-panel-item__input-wrap select:first')
      .select('us-east-1');

      // Intercept the API call and alias it
      cy.wait('@updatePropertyEditorValue', { timeout: 60000 }).its('response.statusCode').should('eq', 200);

      cy.url().then(currentUrl => {
        // Construct a new URL with desired query parameters for selecting 
        // the attribute panel for a known connected component
        let newUrl = new URL(currentUrl);
        newUrl.searchParams.set('s', 'c_'+componentIDB);
        newUrl.searchParams.set('t', 'attributes');
        cy.visit(newUrl.href);
      });

      // Wait for the values to propagate
      cy.wait(3000);

      // Validate that the value has propagated through the system
      cy.get('.attributes-panel-item__input-wrap input.region')
      .should('have.value', 'us-east-1');

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
