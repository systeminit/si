import t from 'tap';
import { expect } from 'chai';
import { z } from 'zod';

import {
  ALLOWED_INPUT_REGEX,
  DOMAIN_FRIENDLY_INPUT_REGEX,
  NAME_REGEX,
  GITHUB_USERNAME_REGEX,
  DISCORD_TAG_REGEX,
  URL_DETECTION_REGEX,
  MAX_LENGTH_STANDARD,
  MAX_LENGTH_EXTENDED,
} from '../src/lib/validation-helpers';

t.test('Validation Helpers - Regex Tests', async () => {
  t.test('ALLOWED_INPUT_REGEX', async (t) => {
    t.test('should allow valid inputs', async () => {
      const validInputs = [
        'Hello World',
        'Test 123',
        "O'Brien",
        'Name-With-Dashes',
        'Name_With_Underscores',
        'Name (with parentheses)',
        'Name+Plus',
        "L'Hôpital",
        'Café',
        'Zürich',
        '',
      ];

      for (const input of validInputs) {
        expect(ALLOWED_INPUT_REGEX.test(input)).to.be.true;
      }
    });

    t.test('should reject inputs with dots (domain names)', async () => {
      const invalidInputs = [
        'example.com',
        'my.domain',
        'user@email.com',
        'https://example.com',
        'http://test.com',
        '/path/to/file',
      ];

      for (const input of invalidInputs) {
        expect(ALLOWED_INPUT_REGEX.test(input)).to.be.false;
      }
    });
  });

  t.test('DOMAIN_FRIENDLY_INPUT_REGEX', async (t) => {
    t.test('should allow valid inputs including domain names', async () => {
      const validInputs = [
        'Hello World',
        'Test 123',
        "O'Brien",
        'Name-With-Dashes',
        'Name_With_Underscores',
        'Name (with parentheses)',
        'Name+Plus',
        "L'Hôpital",
        'Café',
        'Zürich',
        // Domain names (NEW - these should now be allowed)
        'example.com',
        'my.domain',
        'subdomain.example.com',
        'test-site.io',
        'My Workspace (example.com)',
        'Production - api.company.com',
        '',
      ];

      for (const input of validInputs) {
        expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(input)).to.be.true;
      }
    });

    t.test('should reject inputs with URL schemes and @ symbols', async () => {
      const invalidInputs = [
        'user@email.com',
        'https://example.com',
        'http://test.com',
        '/path/to/file',
        'ftp://server.com',
        'mailto:user@example.com',
      ];

      for (const input of invalidInputs) {
        expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(input)).to.be.false;
      }
    });
  });

  t.test('NAME_REGEX', async (t) => {
    t.test('should allow valid names', async () => {
      const validNames = [
        'John Doe',
        "O'Brien",
        'Mary-Jane',
        'José',
        'François',
        'Müller',
        '',
      ];

      for (const name of validNames) {
        expect(NAME_REGEX.test(name)).to.be.true;
      }
    });

    t.test('should reject names with special characters or domain-like patterns', async () => {
      const invalidNames = [
        'john.doe',
        'user@example',
        'name_with_underscore',
        'name+plus',
        'example.com',
        '(parentheses)',
      ];

      for (const name of invalidNames) {
        expect(NAME_REGEX.test(name)).to.be.false;
      }
    });
  });

  t.test('GITHUB_USERNAME_REGEX', async (t) => {
    t.test('should allow valid GitHub usernames', async () => {
      const validUsernames = [
        'john-doe',
        'user123',
        'a',
        'test-user-123',
        'User-Name',
        'x'.repeat(39), // Max length
      ];

      for (const username of validUsernames) {
        expect(GITHUB_USERNAME_REGEX.test(username)).to.be.true;
      }
    });

    t.test('should reject invalid GitHub usernames', async () => {
      const invalidUsernames = [
        '-john', // Starts with hyphen
        'john-', // Ends with hyphen
        'john--doe', // Consecutive hyphens
        'x'.repeat(40), // Too long
        'user@name', // Invalid character
        '', // Empty
      ];

      for (const username of invalidUsernames) {
        expect(GITHUB_USERNAME_REGEX.test(username)).to.be.false;
      }
    });
  });

  t.test('DISCORD_TAG_REGEX', async (t) => {
    t.test('should allow valid Discord usernames', async () => {
      const validTags = [
        'username',
        'user.name',
        'user_name',
        'username#1234',
        'cool.user',
        'ab', // Min length
        'a'.repeat(32), // Max length
      ];

      for (const tag of validTags) {
        expect(DISCORD_TAG_REGEX.test(tag)).to.be.true;
      }
    });

    t.test('should reject invalid Discord usernames', async () => {
      const invalidTags = [
        'discord', // Reserved word
        'here', // Reserved word
        'everyone', // Reserved word
        'user..name', // Consecutive periods
        'a', // Too short
        'a'.repeat(33), // Too long
        'user#123', // Invalid discriminator (not 4 digits)
      ];

      for (const tag of invalidTags) {
        expect(DISCORD_TAG_REGEX.test(tag)).to.be.false;
      }
    });
  });

  t.test('URL_DETECTION_REGEX', async (t) => {
    t.test('should detect URLs', async () => {
      const urls = [
        'https://example.com',
        'http://test.com',
        'www.example.com',
        'example.com',
        'test.io',
        'subdomain.example.co.uk',
      ];

      for (const url of urls) {
        expect(URL_DETECTION_REGEX.test(url)).to.be.true;
      }
    });

    t.test('should not detect non-URLs', async () => {
      const nonUrls = [
        'just some text',
        'no url here',
        'localhost',
        '192.168.1.1',
      ];

      for (const text of nonUrls) {
        expect(URL_DETECTION_REGEX.test(text)).to.be.false;
      }
    });
  });

  t.test('Max Length Validations', async (t) => {
    t.test('should enforce standard character limit for user profile fields', async () => {
      // User profile fields: firstName, lastName, nickname, githubUsername, discordUsername
      const schemaStandard = z.string().max(MAX_LENGTH_STANDARD);

      // Should pass with exactly MAX_LENGTH_STANDARD characters
      const validStandard = 'a'.repeat(MAX_LENGTH_STANDARD);
      expect(schemaStandard.safeParse(validStandard).success).to.be.true;

      // Should fail with MAX_LENGTH_STANDARD + 1 characters
      const invalidStandard = 'a'.repeat(MAX_LENGTH_STANDARD + 1);
      expect(schemaStandard.safeParse(invalidStandard).success).to.be.false;
    });

    t.test('should enforce standard character limit for workspace display names', async () => {
      const schemaStandard = z.string().max(MAX_LENGTH_STANDARD);

      // Should pass with exactly MAX_LENGTH_STANDARD characters
      const validStandard = 'a'.repeat(MAX_LENGTH_STANDARD);
      expect(schemaStandard.safeParse(validStandard).success).to.be.true;

      // Should fail with MAX_LENGTH_STANDARD + 1 characters
      const invalidStandard = 'a'.repeat(MAX_LENGTH_STANDARD + 1);
      expect(schemaStandard.safeParse(invalidStandard).success).to.be.false;
    });

    t.test('should enforce extended character limit for workspace descriptions', async () => {
      const schemaExtended = z.string().max(MAX_LENGTH_EXTENDED);

      // Should pass with exactly MAX_LENGTH_EXTENDED characters
      const validExtended = 'a'.repeat(MAX_LENGTH_EXTENDED);
      expect(schemaExtended.safeParse(validExtended).success).to.be.true;

      // Should fail with MAX_LENGTH_EXTENDED + 1 characters
      const invalidExtended = 'a'.repeat(MAX_LENGTH_EXTENDED + 1);
      expect(schemaExtended.safeParse(invalidExtended).success).to.be.false;
    });

    t.test('should allow domain names within length limits', async () => {
      // Workspace display name: MAX_LENGTH_STANDARD chars with domain
      const displayNameSchema = z.string()
        .max(MAX_LENGTH_STANDARD)
        .regex(DOMAIN_FRIENDLY_INPUT_REGEX);

      const validDisplayName = 'My Production Workspace - example.com';
      expect(displayNameSchema.safeParse(validDisplayName).success).to.be.true;

      // Workspace description: MAX_LENGTH_EXTENDED chars with domain
      const descriptionSchema = z.string()
        .max(MAX_LENGTH_EXTENDED)
        .regex(DOMAIN_FRIENDLY_INPUT_REGEX);

      const validDescription = `This workspace manages api.example.com and handles all production traffic for our services. ${'a'.repeat(400)}`;
      expect(descriptionSchema.safeParse(validDescription).success).to.be.true;
      expect(validDescription.length).to.be.lessThan(MAX_LENGTH_EXTENDED);
    });
  });
});
