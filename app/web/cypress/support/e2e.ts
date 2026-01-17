// ***********************************************************
// This example support/e2e.ts is processed and
// loaded automatically before your test files.
//
// This is a great place to put global configuration and
// behavior that modifies Cypress.
//
// You can change the location of this file or turn off
// automatically serving support files with the
// 'supportFile' configuration option.
//
// You can read more here:
// https://on.cypress.io/configuration
// ***********************************************************

// When a command from ./commands is ready to use, import with `import './commands'` syntax
import './commands';

// Global console logging - enhanced with prefixes
Cypress.on('window:before:load', (win) => {
  const originalLog = win.console.log;
  const originalError = win.console.error;
  const originalWarn = win.console.warn;
  const originalInfo = win.console.info;
  const originalDebug = win.console.debug;

  win.console.log = (...args) => {
    console.log(`[CONSOLE.LOG]`, ...args);
    originalLog.apply(win.console, args);
  };

  win.console.error = (...args) => {
    console.log(`[CONSOLE.ERROR]`, ...args);
    originalError.apply(win.console, args);
  };

  win.console.warn = (...args) => {
    console.log(`[CONSOLE.WARN]`, ...args);
    originalWarn.apply(win.console, args);
  };

  win.console.info = (...args) => {
    console.log(`[CONSOLE.INFO]`, ...args);
    originalInfo.apply(win.console, args);
  };

  win.console.debug = (...args) => {
    console.log(`[CONSOLE.DEBUG]`, ...args);
    originalDebug.apply(win.console, args);
  };
});

// Global network logging with timeout handling
beforeEach(() => {
  cy.intercept('**', (req) => {
    const startTime = Date.now();
    console.log(`[NETWORK REQUEST] ${req.method} ${req.url}`);
    
    if (req.body) {
      console.log(`[REQUEST BODY] ${JSON.stringify(req.body)}`);
    }
    
    if (Object.keys(req.headers).length > 0) {
      console.log(`[REQUEST HEADERS] ${JSON.stringify(req.headers)}`);
    }

    req.continue((res) => {
      const endTime = Date.now();
      const duration = endTime - startTime;
      
      if (res) {
        console.log(`[NETWORK RESPONSE] ${req.method} ${req.url} -> ${res.statusCode} (${duration}ms)`);
        if (res.body) {
          console.log(`[RESPONSE BODY] ${JSON.stringify(res.body)}`);
        }
      } else {
        console.log(`[NETWORK TIMEOUT/ERROR] ${req.method} ${req.url} (${duration}ms) - No response received`);
      }
    });
  }).as('allRequests');
});

// Additional error handling for uncaught exceptions and promise rejections
Cypress.on('uncaught:exception', (err) => {
  console.log(`[UNCAUGHT EXCEPTION] ${err.message}`);
  console.log(`[STACK TRACE] ${err.stack}`);
  return false; // Don't fail the test
});

Cypress.on('window:before:load', (win) => {
  win.addEventListener('unhandledrejection', (event) => {
    console.log(`[UNHANDLED PROMISE REJECTION] ${event.reason}`);
  });
});