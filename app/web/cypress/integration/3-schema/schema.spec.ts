import { ChangeSetService } from "../../../src/service/change_set";
import { firstValueFrom } from "rxjs";

describe("Schema", () => {
  beforeEach(() => {
    cy.visit("/");
    cy.signupAndLogin();
  });

  it("can navigate to the schema editor", () => {
    cy.visit("/");
    cy.getBySel("schema-nav-link")
      .click()
      .then(() => {
        cy.url().should("be.equal", `${Cypress.config("baseUrl")}/schema/list`);
      });
  });

  it("can create a new schema", () => {
    cy.visit("/schema/list");
    cy.window()
      .its("ChangeSetService")
      .then((changeSetService: typeof ChangeSetService) => {
        return firstValueFrom(
          changeSetService.createChangeSet({ changeSetName: "poop" }),
        );
      })
      .then(() => {
        cy.getBySel("schema-new-button").click();
        cy.getBySel("schema-new-form-name").click().type("coffeeCupJapan");
        cy.getBySel("schema-new-form-kind").select("Concrete");
        cy.getBySel("schema-new-form-create-button").click();
        cy.contains("coffeeCupJapan");
      });
  });
});
