// @ts-check
///<reference path="../../global.d.ts" />

// This function can only be run properly if the user is logged in
Cypress.Commands.add("createComponent", (componentName: string) => {
  const log = Cypress.log({
    displayName: "CREATE COMPONENT",
    message: [`🟩 Creating | ${componentName}`],
    // @ts-ignore
    autoEnd: false,
  });
  log.snapshot("before");

  // Find the component in the AssetPalette
  cy.get('.asset-palette div[class="tree-node"]', { timeout: 30000 }).contains(componentName, { matchCase: false }).as('component');

  // Find the canvas to get a location to drag to
  cy.get('.modeling-diagram .konvajs-content').first().as('konvaStage');

  // drag to the canvas
  // TODO(Wendy) - this can never put the component inside of a frame
  cy.dragTo('@component', '@konvaStage');

  // TODO(Wendy) - eventually we should replace this wait! For now this just gives time for the component to be created and load
  cy.wait(5000);

  // Validate that the component was created via the DiagramOutline
  // TODO(Wendy) - this version of createComponent only works for placing components directly onto the canvas, not into frames
  cy.get('.diagram-outline .diagram-outline-node', { timeout: 30000 }).last().contains(componentName, { timeout: 30000, matchCase: false }).should('be.visible');

  log.snapshot("after");
  log.end();
});