export class ComponentAttributes {

  enterInputField(type: string, attributeName: string, text: string) {
    return cy.get(`${type}.${attributeName}`).then(() => {
      if (type === 'input') {
        cy.get(`${type}.${attributeName}`).click().type(text + '{enter}');
      } else if (type === 'select') {
        cy.get(`${type}.${attributeName}`).select(text);
      } else {
        // Handle other types if needed
      }
    });
  }

}
export const componentAttributes = new ComponentAttributes();
