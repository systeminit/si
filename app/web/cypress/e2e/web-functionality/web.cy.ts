// @ts-check
///<reference path="../global.d.ts"/>

const SI_WORKSPACE_URL = Cypress.env('VITE_SI_WORKSPACE_URL') || import.meta.env.VITE_SI_WORKSPACE_URL;

describe('web', () => {
  beforeEach(function () {
    cy.visit("/");
  });

  it('get_summary', () => {
    cy.basicLogin();

    cy.intercept('GET', SI_WORKSPACE_URL + '/api/qualification/get_summary?visibility_change_set_pk=00000000000000000000000000', (req) => {
      // Log the intercepted request URL and response status code
      cy.log(`Request to ${req.url}`, req.response.statusCode);
      // Assert that the status code is 200
      expect(req.response.statusCode).to.eq(200);
    });
  });

  it("create_change_set", () => {
    cy.basicLogin();
    cy.createChangeSet("cool test change set", true);
  });

  it("create_components", () => {
    cy.basicLogin();

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
    cy.basicLogin();
    
    cy.createComponent("region");

    cy.get('[data-testid="explore-grid"]').find(".component").click();

    cy.get('[data-testid="name-input"]').focus();

    cy.focused().type("test name");

    cy.wait(1000);
    cy.press(Cypress.Keyboard.Keys.TAB);
    cy.wait(1000);
    cy.press(Cypress.Keyboard.Keys.TAB);
    cy.wait(1000);

    cy.focused().type("{downArrow}{downArrow}");
    cy.wait(1000);
    cy.press(Cypress.Keyboard.Keys.TAB);

    cy.contains("test name", { timeout: 60000 });
    cy.contains("us-east-1", { timeout: 60000 });

    // abandon the change set to clean up
    cy.abandonCurrentChangeSet();
  });

  it("switch_change_sets", () => {
    cy.basicLogin();

    cy.createChangeSet("change set 1");
    cy.createChangeSet("change set 2");

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
