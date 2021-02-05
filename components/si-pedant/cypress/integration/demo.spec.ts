import {
  uniqueNamesGenerator,
  adjectives,
  colors,
  animals,
} from "unique-names-generator";

interface SiContext {
  billingAccountName: string;
}

context("system initiative", () => {
  before(() => {
    let context: SiContext = {
      billingAccountName: uniqueNamesGenerator({
        dictionaries: [adjectives, colors, animals],
      }),
    };
    cy.wrap(context).as("globalContext");
  });

  it("works", function () {
    cy.viewport(1440, 1440);
    // @ts-ignore - we know that the context is an SiContext, thanks
    cy.get("@globalContext").then((ctx: SiContext) => {
      cy.log("Signup", {
        billingAccountName: ctx.billingAccountName,
      });
      cy.visit("/authenticate/signup", { timeout: 10000 });
      cy.get("[data-testid=billingAccountName]").type(ctx.billingAccountName);
      cy.get("[data-testid=billingAccountDescription]").type("a");
      cy.get("[data-testid=userFullName]").type("a");
      cy.get("[data-testid=userEmail]").type("a");
      cy.get("[data-testid=userPasswordFirst]").type("a");
      cy.get("[data-testid=userPasswordSecond]").type("a");
      cy.get("[aria-label='Sign Up']").click();

      cy.log("Signin");
      cy.url().should("eq", `${Cypress.config().baseUrl}/authenticate/login`, {
        timeout: 20000,
      });
      cy.get("[data-testid=billingAccountName]").type(ctx.billingAccountName);
      cy.get("[data-testid=userEmail]").type("a");
      cy.get("[data-testid=userPassword]").type("a");
      cy.get("[aria-label='Login']").click();

      cy.log("Create application");
      cy.url().should("match", /^.+\/o\/.+\/w\/.+\/a$/, {
        timeout: 20000,
      });
      cy.get("[aria-label='New Application']").click();
      cy.get("[data-testid=applicationName]").type("amon amarth");
      cy.get("[aria-label='Create']").click();
    });
  });
});
