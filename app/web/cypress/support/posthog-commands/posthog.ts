// @ts-check
///<reference path="../../global.d.ts" />

// Note: this function leaves you on a blank page, so you must call cy.visit()
// afterwards, before continuing with your test.
Cypress.Commands.add("sendPosthogEvent", (event: string, eventKey: string, eventData: string) => {
    const log = Cypress.log({
      displayName: "SENDING POSTHOG EVENT",
      message: [`Event: ${event} / ${eventKey} : ${eventData}`],
      // @ts-ignore
      autoEnd: false,
    });
  
    cy.window().then((win) => {
        ;(win as any).posthog?.capture(event, { eventKey: eventData })
    })

    log.end();
  });