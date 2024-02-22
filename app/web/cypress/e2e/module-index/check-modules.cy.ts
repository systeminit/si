// @ts-check
///<reference path="../global.d.ts"/>

describe('Module Index Validate mdoules', () => {
    it('Should have at least 40 modules', () => {
      cy.request('https://module-index.systeminit.com/builtins').then((response) => {
        // Ensure that the response status is 200
        expect(response.status).to.eq(200);
  
        // Parse the response body as JSON
        const responseBody = response.body;
  
        // Ensure that the response body contains a 'modules' property
        expect(responseBody).to.have.property('modules');

        // Get the number of modules
        const numberOfModules = responseBody.modules.length;

        // Log the number of modules
        cy.log(`Number of modules found to be: ${numberOfModules}`);

        // Ensure that there are at least 50 modules
        expect(numberOfModules).to.be.at.least(40);
      });
    });
  });