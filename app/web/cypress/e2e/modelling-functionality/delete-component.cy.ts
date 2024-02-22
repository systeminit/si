// @ts-check
///<reference path="../global.d.ts"/>

describe('Delete Components', () => {
  beforeEach(function () {
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
  });

  it('should pick up an AWS Credential add then delete it from the diagram', () => {
    cy.visit('/')
    cy.contains('Create change set', { timeout: 10000 })
      .should('be.visible')
      .click();

    // Find the AWS Credential
    cy.get('[data-cy="asset_card', { timeout: 10000 }).contains('AWS Credential', { timeout: 10000 }).should('be.visible').as('awsCred')

    // Find the canvas to get a location to drag to
    cy.get('canvas').first().as('konvaStage');

    // Drag to the canvas
    cy.dragTo('@awsCred', '@konvaStage');

    // Check to make sure a component has been added to the outliner
    cy.get('[class="component-outline-node"]', { timeout: 10000 }).
      contains('AWS Credential', { timeout: 10000 })
      .should('be.visible')
      .rightclick();

    // Click the second dropdown menu item
    cy.get('#dropdown-menu-item-2').click();

    // Click the destroy button
    cy.get('button.vbutton.--variant-solid.--size-md.--tone-destructive')
      .click();

    //check to make sure a component has been added to the outliner
    cy.get('[class="component-outline-node"]', { timeout: 10000 }).contains('AWS Credential', { timeout: 10000 }).should('be.visible');

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

