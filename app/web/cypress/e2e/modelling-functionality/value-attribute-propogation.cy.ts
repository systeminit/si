// @ts-check
///<reference path="../global.d.ts"/>

Cypress._.times(import.meta.env.VITE_SI_CYPRESS_MULTIPLIER ? import.meta.env.VITE_SI_CYPRESS_MULTIPLIER : 1, () => {
  describe('Value Propogation', () => {
    beforeEach(function () {
      cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
    });

    it('change value on component A and check it propogates to component B', () => {

      // Go to the Synthetic Workspace
      cy.visit(import.meta.env.VITE_SI_WORKSPACE_URL + '/w/' + import.meta.env.VITE_SI_WORKSPACE_ID + '/head')

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', 'Change Set 1');
      
      cy.get('#vorm-input-3').clear().type(import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

      cy.get('#vorm-input-3', { timeout: 30000 }).should('have.value', import.meta.env.VITE_UUID ? import.meta.env.VITE_UUID: "local");

      cy.contains('Create change set', { timeout: 30000 }).click();

      // Give time to redirect onto the new changeset
      cy.url().should('not.include', 'head', { timeout: 10000 });

      cy.url().then(currentUrl => {
        // Construct a new URL with desired query parameters for selecting 
        // the attribute panel for a known component
        let newUrl = new URL(currentUrl);
        newUrl.searchParams.set('s', import.meta.env.VITE_SI_PROPAGATION_COMPONENT_A);
        newUrl.searchParams.set('t', 'attributes');
      
        // Visit the new URL
        cy.visit(newUrl.href);
      });

      // Give the page a few seconds to load
      cy.wait(2000);

      // Generate a random number between 1 and 100 to insert into the 
      // attribute value for Integer
      const randomNumber = Math.floor(Math.random() * 100) + 1;

      // Find the attribute for the Integer Input
      cy.get('.attributes-panel-item__input-wrap input[type="number"]')
      .clear()
      .type(randomNumber.toString() + '{enter}') // type the new value

      // This will have auto-directed us onto a changeset, give it a few seconds
      // to load up
      cy.wait(3000)

      cy.url().then(currentUrl => {
        // Construct a new URL with desired query parameters for selecting 
        // the attribute panel for a known connected component
        let newUrl = new URL(currentUrl);
        newUrl.searchParams.set('s', import.meta.env.VITE_SI_PROPAGATION_COMPONENT_B);
        newUrl.searchParams.set('t', 'attributes');
        cy.visit(newUrl.href);
      });

      // Wait for the values to propagate
      cy.wait(30000);

      // Validate that the value has propogated through the system
      cy.get('.attributes-panel-item__input-wrap input[type="number"]')
      .should('have.value', randomNumber.toString(), { timeout: 30000 });

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