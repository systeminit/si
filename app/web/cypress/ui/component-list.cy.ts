// @ts-check
///<reference path="../global.d.ts"/>

const AUTH0_USERNAME = Cypress.env('VITE_AUTH0_USERNAME') || import.meta.env.VITE_AUTH0_USERNAME;
const AUTH0_PASSWORD = Cypress.env('VITE_AUTH0_PASSWORD') || import.meta.env.VITE_AUTH0_PASSWORD;
const SI_WORKSPACE_URL = Cypress.env('VITE_SI_WORKSPACE_URL') || import.meta.env.VITE_SI_WORKSPACE_URL;
const SI_WORKSPACE_ID = Cypress.env('VITE_SI_WORKSPACE_ID') || import.meta.env.VITE_SI_WORKSPACE_ID;
const UUID = Cypress.env('VITE_UUID') || import.meta.env.VITE_UUID || "local";

describe('web', () => {
  beforeEach(function () {
    cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);
  });

  it('get_tile_list', () => {
    // Go to the Synthetic Workspace
    cy.visit({
        url: SI_WORKSPACE_URL + '/w/' + SI_WORKSPACE_ID + '/head',
        failOnStatusCode: false
    });
    cy.on('uncaught:exception', (e) => {
      console.log(e);
      return false;
    });
    cy.sendPosthogEvent(Cypress.currentTest.titlePath.join("/"), "test_uuid", UUID);
    cy.get('.scrollable.tilegrid.grow')
    .children('.component.tile.cursor-pointer')
    .its('length')
    .then(count => {
        cy.log(`Tile count: ${count}`);
        expect(count).to.be.greaterThan(0);
    });
  });
});
