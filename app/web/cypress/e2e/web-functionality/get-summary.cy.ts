// @ts-check
///<reference path="../global.d.ts"/>

const SI_WORKSPACE_URL = Cypress.env('VITE_SI_WORKSPACE_URL') || import.meta.env.VITE_SI_WORKSPACE_URL;

describe('get_summary', () => {
  it('get_summary', () => {
    cy.basicLogin();

    cy.intercept('GET', SI_WORKSPACE_URL + '/api/qualification/get_summary?visibility_change_set_pk=00000000000000000000000000', (req) => {
      // Log the intercepted request URL and response status code
      cy.log(`Request to ${req.url}`, req.response.statusCode);
      // Assert that the status code is 200
      expect(req.response.statusCode).to.eq(200);
    });
  });
});
