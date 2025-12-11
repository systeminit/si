/// <reference types="cypress" />
///<reference path="../global.d.ts" />

const env = (key: string) => {
  const valueFromCypress = Cypress.env(key);
  if (typeof valueFromCypress === 'string') {
    return valueFromCypress;
  }

  const metaEnv = (import.meta as unknown as { env?: Record<string, unknown> }).env;
  const fallback = metaEnv?.[key];
  return typeof fallback === 'string' ? fallback : undefined;
};

const SI_WORKSPACE_ID = env('VITE_SI_WORKSPACE_ID');
const AUTH_API_URL = env('VITE_AUTH_API_URL');
const AUTH_PORTAL_URL = env('VITE_AUTH_PORTAL_URL');
const AUTH0_USERNAME = env('VITE_AUTH0_USERNAME');
const AUTH0_PASSWORD = env('VITE_AUTH0_PASSWORD');
const UUID = env('VITE_UUID') || 'local';

const selectDefaultSubscriptionsButton = () => {
  const selector = '[data-testid="default-subscriptions-button"]';
  cy.get('body').then(($body) => {
    if ($body.find(selector).length > 0) {
      cy.get(selector).click();
    } else {
      cy.contains('button', 'See default subscriptions', { matchCase: false }).click();
    }
  });
};

describe('Default Subscriptions Toggle', () => {
  beforeEach(() => {
    if (!AUTH0_USERNAME || !AUTH0_PASSWORD || !AUTH_API_URL || !AUTH_PORTAL_URL) {
      throw new Error('Missing Auth0 or workspace portal configuration for default subscriptions test');
    }
    if (!SI_WORKSPACE_ID) {
      throw new Error('Missing workspace ID for default subscriptions test');
    }

    cy.loginToAuth0(AUTH0_USERNAME, AUTH0_PASSWORD);

    cy.visit(`${AUTH_PORTAL_URL}/dashboard`);
    cy.sendPosthogEvent(Cypress.currentTest.titlePath.join('/'), 'test_uuid', UUID);

    cy.wait(5000);

    cy.get(`a[href="${AUTH_API_URL}/workspaces/${SI_WORKSPACE_ID}/go"]`, {
      timeout: 60000,
    })
      .should('be.visible')
      .invoke('removeAttr', 'target')
      .click();
    cy.on('uncaught:exception', (e) => {
      // eslint-disable-next-line no-console
      console.log(e);
      return false;
    });
    cy.sendPosthogEvent(
      Cypress.currentTest.titlePath.join('/'),
      'test_uuid',
      UUID,
    );

    cy.get('[data-testid="left-column-new-hotness-explore"]', {
      timeout: 10000,
    }).should('exist');
  });

  it('toggles default subscriptions view and updates URL query', () => {
    selectDefaultSubscriptionsButton();
    cy.location('search').should((search) => {
      expect(search).to.include('defaultSubscriptions=1');
    });

    selectDefaultSubscriptionsButton();
    cy.location('search').should((search) => {
      expect(search).not.to.include('defaultSubscriptions=1');
    });
  });
});
