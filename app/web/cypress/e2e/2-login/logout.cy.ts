describe("Logout", () => {
  beforeEach(() => {
    cy.visit("/");
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
  });

  it("lets the user log out", () => {
    cy.visit("/");
    cy.contains('Create change set', { timeout: 10000 }).should('be.visible').click();
    cy.get('[aria-label="Profile"]').should('exist').click();
    cy.get('#dropdown-menu-item-1').should('exist').should('be.visible').click({ force: true });
    cy.url().should("contain", import.meta.env.VITE_AUTH_PORTAL_URL + '/logout');

  });
});
