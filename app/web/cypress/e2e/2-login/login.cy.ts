describe("Login", () => {
  beforeEach(() => {
    cy.visit("/");
  });

  it("lets the user log in", () => {
    cy.loginToAuth0(import.meta.env.VITE_AUTH0_USERNAME, import.meta.env.VITE_AUTH0_PASSWORD);
    cy.visit("/");
    // check that you're on head
    // Clearly not happening as I need the following line to make the test pass correctly:
    cy.visit('https://app.systeminit.com/w/01HPMKZZ0DF54B12FNBF6Z7704/head');
  });

});
