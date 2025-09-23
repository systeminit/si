// @ts-check
///<reference path="../../global.d.ts" />

// Note: this function leaves you on a blank page, so you must call cy.visit()
// afterwards, before continuing with your test.
Cypress.Commands.add("loginToAuth0", (username: string, password: string) => {
  const log = Cypress.log({
    displayName: "AUTH0 LOGIN",
    message: [`🔐 Authenticating | ${username}`],
    // @ts-ignore
    autoEnd: false,
  });
  log.snapshot("before");

  const args = { username, password };
  cy.session(
    `auth0-${username}`,
    () => {

      // App landing page redirects to Auth0.
      cy.visit("/");
      cy.log('At homepage')

      cy.url({ timeout: 60000 }).should("contain", import.meta.env.VITE_AUTH0_DOMAIN);

      cy.visit("/");

      // Login on Auth0.
      cy.origin(import.meta.env.VITE_AUTH0_DOMAIN, { args }, ({ username, password }) => {
        cy.get("input#username").type(username);
        cy.contains('Continue').click();
        cy.get("input#password").type(password).type('{enter}');
      });

      // Ensure Auth0 has redirected us back to the auth portal.
      cy.url({ timeout: 60000 }).should("contain", import.meta.env.VITE_AUTH_PORTAL_URL);

      // Push onto the workspace requested
      cy.visit({
        url: import.meta.env.VITE_AUTH_API_URL + '/workspaces/' + import.meta.env.VITE_SI_WORKSPACE_ID + '/go',
        failOnStatusCode: false
      });

    },
    // {
    //   validate: () => {
    //     // Validate presence of access token in localStorage.
    //     cy.window().its("localStorage").invoke("getItem", "si-auth").should("exist");
    //   },
    // }
  );

  log.snapshot("after");
  log.end();
});
