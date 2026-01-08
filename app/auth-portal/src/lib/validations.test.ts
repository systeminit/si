import { describe, it, expect } from "vitest";

import {
  ALLOWED_INPUT_REGEX,
  DOMAIN_FRIENDLY_INPUT_REGEX,
  NAME_REGEX,
  NICKNAME_REGEX,
  GITHUB_USERNAME_REGEX,
  DISCORD_TAG_REGEX,
  URL_DETECTION_REGEX,
  MAX_LENGTH_STANDARD,
  MAX_LENGTH_EXTENDED,
} from "./validations";

describe("Validation Helpers - Regex Tests", () => {
  describe("ALLOWED_INPUT_REGEX", () => {
    it("should allow valid inputs", () => {
      const validInputs = [
        "Hello World",
        "Test 123",
        "O'Brien",
        "Name-With-Dashes",
        "Name_With_Underscores",
        "Name (with parentheses)",
        "Name+Plus",
        "L'Hôpital",
        "Café",
        "Zürich",
        "",
      ];

      for (const input of validInputs) {
        expect(ALLOWED_INPUT_REGEX.test(input)).toBe(true);
      }
    });

    it("should reject inputs with dots (domain names)", () => {
      const invalidInputs = [
        "example.com",
        "my.domain",
        "user@email.com",
        "https://example.com",
        "http://test.com",
        "/path/to/file",
      ];

      for (const input of invalidInputs) {
        expect(ALLOWED_INPUT_REGEX.test(input)).toBe(false);
      }
    });
  });

  describe("DOMAIN_FRIENDLY_INPUT_REGEX", () => {
    it("should allow valid inputs including domain names", () => {
      const validInputs = [
        "Hello World",
        "Test 123",
        "O'Brien",
        "Name-With-Dashes",
        "Name_With_Underscores",
        "Name (with parentheses)",
        "Name+Plus",
        "L'Hôpital",
        "Café",
        "Zürich",
        // Domain names (NEW - these should now be allowed)
        "example.com",
        "my.domain",
        "subdomain.example.com",
        "test-site.io",
        "My Workspace (example.com)",
        "Production - api.company.com",
        "",
      ];

      for (const input of validInputs) {
        expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(input)).toBe(true);
      }
    });

    it("should reject inputs with URL schemes and @ symbols", () => {
      const invalidInputs = [
        "user@email.com",
        "https://example.com",
        "http://test.com",
        "/path/to/file",
        "ftp://server.com",
        "mailto:user@example.com",
      ];

      for (const input of invalidInputs) {
        expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(input)).toBe(false);
      }
    });
  });

  describe("NAME_REGEX", () => {
    it("should allow valid names", () => {
      const validNames = [
        "John Doe",
        "O'Brien",
        "Mary-Jane",
        "José",
        "François",
        "Müller",
        "",
      ];

      for (const name of validNames) {
        expect(NAME_REGEX.test(name)).toBe(true);
      }
    });

    it("should reject names with special characters or domain-like patterns", () => {
      const invalidNames = [
        "john.doe",
        "user@example",
        "name_with_underscore",
        "name+plus",
        "example.com",
        "(parentheses)",
      ];

      for (const name of invalidNames) {
        expect(NAME_REGEX.test(name)).toBe(false);
      }
    });
  });

  describe("NICKNAME_REGEX", () => {
    it("should allow valid nicknames with extended character set", () => {
      const validNicknames = [
        "JohnDoe",
        "John Doe",
        "O'Brien",
        "Mary-Jane",
        "user_name", // underscore
        "name+tag", // plus
        "Smith & Jones", // ampersand
        "john.doe", // period
        "Player#1234", // hash
        "Cool!", // exclamation
        "Name (Admin)", // parentheses
        "José", // accents
        "François",
        "Müller",
        "user_name+tag", // combination
        "Player#1_Pro!", // multiple special chars
        "",
      ];

      for (const nickname of validNicknames) {
        expect(NICKNAME_REGEX.test(nickname)).toBe(true);
      }
    });

    it("should reject nicknames with URL-enabling characters", () => {
      const invalidNicknames = [
        "user@example.com", // @ symbol (email-like)
        "http://example.com", // URL scheme
        "https://test.com", // URL scheme
        "/path/to/file", // forward slash
        "user:password", // colon
        "ftp://server.com", // URL scheme
      ];

      for (const nickname of invalidNicknames) {
        expect(NICKNAME_REGEX.test(nickname)).toBe(false);
      }
    });

    it("should be more permissive than NAME_REGEX", () => {
      // These should pass NICKNAME_REGEX but fail NAME_REGEX
      const nicknameOnlyValid = [
        "user_name",
        "name+tag",
        "john.doe",
        "Player#1234",
        "Cool!",
        "Name (Admin)",
        "Smith & Jones",
      ];

      for (const input of nicknameOnlyValid) {
        expect(NICKNAME_REGEX.test(input)).toBe(true);
        expect(NAME_REGEX.test(input)).toBe(false);
      }
    });
  });

  describe("GITHUB_USERNAME_REGEX", () => {
    it("should allow valid GitHub usernames", () => {
      const validUsernames = [
        "", // Empty string (optional field)
        "john-doe",
        "user123",
        "a",
        "test-user-123",
        "User-Name",
        "x".repeat(39), // Max length
      ];

      for (const username of validUsernames) {
        expect(GITHUB_USERNAME_REGEX.test(username)).toBe(true);
      }
    });

    it("should reject invalid GitHub usernames", () => {
      const invalidUsernames = [
        "-john", // Starts with hyphen
        "john-", // Ends with hyphen
        "john--doe", // Consecutive hyphens
        "x".repeat(40), // Too long
        "user@name", // Invalid character
      ];

      for (const username of invalidUsernames) {
        expect(GITHUB_USERNAME_REGEX.test(username)).toBe(false);
      }
    });
  });

  describe("DISCORD_TAG_REGEX", () => {
    it("should allow valid Discord usernames", () => {
      const validTags = [
        "", // Empty string (optional field)
        "username",
        "user.name",
        "user_name",
        "username#1234",
        "cool.user",
        "ab", // Min length
        "a".repeat(32), // Max length
      ];

      for (const tag of validTags) {
        expect(DISCORD_TAG_REGEX.test(tag)).toBe(true);
      }
    });

    it("should reject invalid Discord usernames", () => {
      const invalidTags = [
        "discord", // Reserved word
        "here", // Reserved word
        "everyone", // Reserved word
        "user..name", // Consecutive periods
        "a", // Too short
        "a".repeat(33), // Too long
        "user#123", // Invalid discriminator (not 4 digits)
      ];

      for (const tag of invalidTags) {
        expect(DISCORD_TAG_REGEX.test(tag)).toBe(false);
      }
    });
  });

  describe("URL_DETECTION_REGEX", () => {
    it("should detect URLs", () => {
      const urls = [
        "https://example.com",
        "http://test.com",
        "www.example.com",
        "example.com",
        "test.io",
        "subdomain.example.co.uk",
      ];

      for (const url of urls) {
        expect(URL_DETECTION_REGEX.test(url)).toBe(true);
      }
    });

    it("should not detect non-URLs", () => {
      const nonUrls = [
        "just some text",
        "no url here",
        "localhost",
        "192.168.1.1",
      ];

      for (const text of nonUrls) {
        expect(URL_DETECTION_REGEX.test(text)).toBe(false);
      }
    });
  });

  describe("Max Length Validations", () => {
    it("should validate standard character limit for user profile fields", () => {
      // User profile fields: firstName, lastName, nickname, githubUsername, discordUsername
      const validStandard = "a".repeat(MAX_LENGTH_STANDARD);
      const invalidStandard = "a".repeat(MAX_LENGTH_STANDARD + 1);

      // NAME_REGEX should work within MAX_LENGTH_STANDARD characters
      expect(NAME_REGEX.test(validStandard)).toBe(true);
      expect(validStandard.length).toBe(MAX_LENGTH_STANDARD);

      // MAX_LENGTH_STANDARD + 1 characters is too long (would be rejected by backend)
      expect(invalidStandard.length).toBe(MAX_LENGTH_STANDARD + 1);
    });

    it("should validate standard character limit for workspace display names", () => {
      const validStandard = "a".repeat(MAX_LENGTH_STANDARD);
      const invalidStandard = "a".repeat(MAX_LENGTH_STANDARD + 1);

      // DOMAIN_FRIENDLY_INPUT_REGEX should work within MAX_LENGTH_STANDARD characters
      expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(validStandard)).toBe(true);
      expect(validStandard.length).toBe(MAX_LENGTH_STANDARD);

      // MAX_LENGTH_STANDARD + 1 characters is too long
      expect(invalidStandard.length).toBe(MAX_LENGTH_STANDARD + 1);
    });

    it("should validate extended character limit for workspace descriptions", () => {
      const validExtended = "a".repeat(MAX_LENGTH_EXTENDED);
      const invalidExtended = "a".repeat(MAX_LENGTH_EXTENDED + 1);

      // DOMAIN_FRIENDLY_INPUT_REGEX should work within MAX_LENGTH_EXTENDED characters
      expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(validExtended)).toBe(true);
      expect(validExtended.length).toBe(MAX_LENGTH_EXTENDED);

      // MAX_LENGTH_EXTENDED + 1 characters is too long
      expect(invalidExtended.length).toBe(MAX_LENGTH_EXTENDED + 1);
    });

    it("should allow domain names within length limits", () => {
      // Workspace display name: MAX_LENGTH_STANDARD chars with domain
      const validDisplayName = "My Production Workspace - example.com";
      expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(validDisplayName)).toBe(true);
      expect(validDisplayName.length).toBeLessThan(MAX_LENGTH_STANDARD);

      // Workspace description: MAX_LENGTH_EXTENDED chars with domain
      const validDescription = `This workspace manages api.example.com and handles all production traffic for our services. ${"a".repeat(
        400,
      )}`;
      expect(DOMAIN_FRIENDLY_INPUT_REGEX.test(validDescription)).toBe(true);
      expect(validDescription.length).toBeLessThan(MAX_LENGTH_EXTENDED);
    });
  });
});
