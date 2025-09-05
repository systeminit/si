// @ts-check
///<reference path="../global.d.ts"/>

describe('web', () => {
  it("create_change_set", () => {
    cy.basicLogin(true);
    cy.createChangeSet("cool test change set", true);
  });

  it("create_components", () => {
    cy.basicLogin(true);

    // A low number to avoid the virtualizer
    const numberOfComponents = 5;

    // create N components
    for (let n = 0; n < numberOfComponents; n++) {
      cy.createComponent();
    }

    // make sure there are N components on the grid
    cy.get('[data-testid="explore-grid"]').find(".component").its("length").should("eq", numberOfComponents);

    // abandon the change set to clean up
    cy.abandonCurrentChangeSet();
  });

  it("edit_component", () => {
    cy.basicLogin(true);
    
    cy.createComponent("region");

    cy.get('[data-testid="explore-grid"]').find(".component").click();

    cy.get('[data-testid="name-input"]').focus();

    cy.focused().type("test name");

    cy.press(Cypress.Keyboard.Keys.TAB);
    cy.press(Cypress.Keyboard.Keys.TAB);

    cy.focused().type("{downArrow}{downArrow}");

    cy.press(Cypress.Keyboard.Keys.TAB);

    cy.contains("test name", { timeout: 60000 });
    cy.contains("us-east-1", { timeout: 60000 });

    // abandon the change set to clean up
    cy.abandonCurrentChangeSet();
  });

  it("switch_change_sets", () => {
    cy.basicLogin(true);

    cy.createChangeSet("cool change set 1");
    cy.createChangeSet("cool change set 2");

    const menu = cy.contains("change set 2", { timeout: 10000 });

    menu.click();
    cy.contains("HEAD", { timeout: 10000 }).parent().parent().click();
    menu.click();
    cy.contains("change set 2", { timeout: 10000 }).parent().parent().click();

    // abandon the change sets to clean up
    cy.abandonCurrentChangeSet();
    menu.click();
    cy.contains("change set 1", { timeout: 10000 }).parent().parent().click();
    cy.abandonCurrentChangeSet();
  });
});
