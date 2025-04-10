// @ts-check
///<reference path="../global.d.ts"/>

describe('webworkertest', () => {
  it('hello web worker', () => {
    cy.visit('http://localhost:8080/webworkertest.html');
      
    // wait for it to run
    cy.wait(2000);
    // make sure its done!
    cy.get("#timestamp").should('not.be.empty');
    // make sure there are no errors
    cy.get("#errors > ul").children().should('have.length', 0);
  });
});