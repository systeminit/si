// @ts-check
///<reference path="../../global.d.ts" />

// This function can only be run properly if the user is logged in and on the Explore page in the new UI
Cypress.Commands.add("createComponent", (componentName?: string, closeComponent=true) => {
  const messageText = componentName ? `"${componentName}"` : 'a component';
  const log = Cypress.log({
    displayName: "CREATE COMPONENT",
    message: [`🟩 Creating ${messageText}`],
    // @ts-ignore
    autoEnd: false,
  });
  log.snapshot("before");

  cy.get('body').type("n");
  cy.get('[data-testid="add-component-search"]', { timeout: 60000 }).find('input').focus();
  if (componentName) {
    cy.focused().type(componentName);
    cy.wait(3000);
  }
  cy.focused().type("{downArrow}{enter}");
  cy.get('[data-testid="component-name-section"]', { timeout: 60000 });

  if (closeComponent) {
    cy.focused().type("{esc}");
    cy.get('body').type("{esc}");
  }

  log.snapshot("after");
  log.end();
});