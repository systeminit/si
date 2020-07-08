export function vuex(): Cypress.Chainable<VuexStore> {
  return cy.get("div#app").then((app) => {
    console.log(app);
    // @ts-ignore - we know it is there, even if you don't
    const vuex: VuexStore = app[0].__vue__.$store;
    return vuex;
  });
}
