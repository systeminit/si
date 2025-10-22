/**
 * Prybar - A custom REPL for Deno CLI applications
 *
 * This module provides a full-featured Read-Eval-Print Loop (REPL) with advanced
 * line editing capabilities, command history, and dynamic variable injection. It
 * implements raw terminal mode input handling to provide an interactive console
 * experience similar to Node.js's REPL or Python's interactive interpreter.
 *
 * ## Features
 *
 * - **Line Editing**: Full cursor movement and text manipulation
 *   - Arrow keys (Left/Right) for cursor navigation
 *   - Home/End keys for jumping to line boundaries
 *   - Backspace/Delete for character removal
 *   - Insert characters at any cursor position
 *
 * - **Command History**: Navigate previously entered commands
 *   - Up/Down arrows to cycle through history
 *   - Persistent history for the current session
 *
 * - **Variable Injection**: Access to context variables
 *   - All properties from the context object are available as variables
 *   - Supports both synchronous and asynchronous operations
 *
 * - **Expression Evaluation**: Smart evaluation strategy
 *   - First attempts to evaluate input as an expression (returns value)
 *   - Falls back to statement evaluation if expression parsing fails
 *   - Displays results of expressions automatically
 *
 * - **Built-in Commands**:
 *   - `help()` - Display available commands and variables
 *   - `close()` - Exit the REPL
 *
 * - **Exit Options**:
 *   - `close()` command
 *   - Ctrl+C (interrupt signal)
 *   - Ctrl+D (EOF signal) on empty line
 *
 * ## Technical Implementation
 *
 * The REPL uses raw terminal mode (`Deno.stdin.setRaw(true)`) to capture
 * individual keystrokes and ANSI escape sequences. This allows it to:
 * - Process special keys (arrows, Home, End, Delete)
 * - Implement cursor positioning with ANSI escape codes
 * - Provide immediate visual feedback without line buffering
 *
 * ## Use Cases
 *
 * - Interactive debugging and inspection during CLI application execution
 * - Live exploration of application state and context
 * - Testing and experimenting with API calls in development
 * - Educational and demonstration purposes
 *
 * @module
 */

/**
 * Starts a custom REPL with access to provided variables.
 *
 * This function initializes an interactive REPL session where all properties
 * from the `context` object are injected as variables into the evaluation scope.
 * The REPL continues running until the user exits via `close()`, Ctrl+C, or
 * Ctrl+D.
 *
 * ## Behavior
 *
 * The REPL operates in raw terminal mode, reading input character-by-character
 * to provide immediate feedback and line editing capabilities. It maintains:
 * - Current line buffer with cursor position tracking
 * - Command history for the session
 * - Visual line editing with ANSI escape sequences
 *
 * ## Input Handling
 *
 * **Control Characters:**
 * - `Ctrl+C` (code 3): Immediately exit the REPL
 * - `Ctrl+D` (code 4): Exit if current line is empty (standard EOF behavior)
 * - `Enter` (codes 13, 10): Evaluate the current line
 * - `Backspace` (codes 127, 8): Delete character before cursor
 *
 * **Escape Sequences (Arrow Keys, etc.):**
 * - `Up Arrow` (ESC[A): Navigate to previous command in history
 * - `Down Arrow` (ESC[B): Navigate to next command in history
 * - `Right Arrow` (ESC[C): Move cursor right one character
 * - `Left Arrow` (ESC[D): Move cursor left one character
 * - `Home` (ESC[H or ESC[1~): Move cursor to beginning of line
 * - `End` (ESC[F or ESC[4~): Move cursor to end of line
 * - `Delete` (ESC[3~): Delete character at cursor position
 *
 * ## Evaluation Strategy
 *
 * When a line is submitted, the REPL attempts to evaluate it in two ways:
 *
 * 1. **Expression Evaluation**: Wraps input in `return ()` and evaluates.
 *    If successful, the return value is displayed.
 * 2. **Statement Evaluation**: If expression evaluation fails, evaluates
 *    the input as a statement without wrapping.
 *
 * This allows both `2 + 2` (expression) and `const x = 5; console.log(x)`
 * (statement) to work naturally.
 *
 * ## Special Features
 *
 * **Logger Integration**: If the context contains a `logger` object with an
 * `info` method, it will be used to log session end messages.
 *
 * **Async Support**: All evaluations are performed in an async context,
 * allowing the use of `await` and promises directly in the REPL.
 *
 * @param context - Object containing variables to make available in the REPL.
 *   Each property becomes a variable accessible during evaluation.
 * @param options - Optional configuration
 * @param options.prompt - The prompt string to display (defaults to "prybar> ")
 *
 * @returns A promise that resolves when the REPL session ends
 *
 * @example Basic usage with default prompt
 * ```ts
 * import { repl } from "./prybar.ts";
 *
 * await repl({
 *   ctx: myContext,
 *   db: database,
 *   utils: utilityFunctions,
 * });
 * // User can now access ctx, db, and utils as variables
 * // Example REPL session:
 * // prybar> ctx.someProperty
 * // prybar> await db.query("SELECT * FROM users")
 * // prybar> utils.formatDate(new Date())
 * ```
 *
 * @example Custom prompt
 * ```ts
 * await repl({
 *   ctx: myContext,
 *   db: database,
 * }, { prompt: "my-app> " });
 * ```
 *
 * @example With logger integration
 * ```ts
 * import { Logger } from "./logger.ts";
 *
 * const logger = new Logger();
 * await repl({
 *   logger,  // Will be used to log "REPL session ended"
 *   config: myConfig,
 *   api: apiClient,
 * });
 * ```
 */
export async function repl(
  context: Record<string, unknown>,
  options?: { prompt?: string },
) {
  const prompt = options?.prompt ?? "prybar> ";

  const history: string[] = [];
  let historyIndex = -1;

  // Set stdin to raw mode for character-by-character input
  Deno.stdin.setRaw(true);

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  try {
    // Write welcome message
    console.log("Type help() for help, close() or Ctrl+C/Ctrl+D to exit");

    // Write initial prompt
    await Deno.stdout.write(encoder.encode(prompt));

    let currentLine = "";
    let cursorPosition = 0;

    const buffer = new Uint8Array(1024);

    while (true) {
      const n = await Deno.stdin.read(buffer);
      if (n === null) break;

      const input = decoder.decode(buffer.subarray(0, n));

      // Handle different input types
      for (let i = 0; i < input.length; i++) {
        const char = input[i];
        const code = input.charCodeAt(i);

        // Handle Ctrl+C
        if (code === 3) {
          await Deno.stdout.write(encoder.encode("\n"));
          // Use logger if available in context
          if (
            "logger" in context &&
            typeof context.logger === "object" &&
            context.logger !== null &&
            "info" in context.logger
          ) {
            (context.logger as { info: (msg: string) => void }).info(
              "REPL session ended",
            );
          }
          return;
        }

        // Handle Ctrl+D (EOF)
        if (code === 4) {
          if (currentLine.length === 0) {
            await Deno.stdout.write(encoder.encode("\n"));
            // Use logger if available in context
            if (
              "logger" in context &&
              typeof context.logger === "object" &&
              context.logger !== null &&
              "info" in context.logger
            ) {
              (context.logger as { info: (msg: string) => void }).info(
                "REPL session ended",
              );
            }
            return;
          }
        }

        // Handle escape sequences (arrow keys, etc.)
        if (code === 27 && i + 2 < input.length) {
          if (input[i + 1] === "[") {
            const escapeCode = input[i + 2];

            // Up arrow - previous history
            if (escapeCode === "A") {
              if (history.length > 0 && historyIndex < history.length - 1) {
                historyIndex++;
                // Clear current line content (not the prompt)
                await clearLineContent(currentLine.length, cursorPosition);
                // Set line from history
                currentLine = history[history.length - 1 - historyIndex];
                cursorPosition = currentLine.length;
                await Deno.stdout.write(encoder.encode(currentLine));
              }
              i += 2;
              continue;
            }

            // Down arrow - next history
            if (escapeCode === "B") {
              if (historyIndex > 0) {
                historyIndex--;
                // Clear current line content (not the prompt)
                await clearLineContent(currentLine.length, cursorPosition);
                // Set line from history
                currentLine = history[history.length - 1 - historyIndex];
                cursorPosition = currentLine.length;
                await Deno.stdout.write(encoder.encode(currentLine));
              } else if (historyIndex === 0) {
                historyIndex = -1;
                // Clear current line content (not the prompt)
                await clearLineContent(currentLine.length, cursorPosition);
                currentLine = "";
                cursorPosition = 0;
              }
              i += 2;
              continue;
            }

            // Right arrow - move cursor right
            if (escapeCode === "C") {
              if (cursorPosition < currentLine.length) {
                cursorPosition++;
                await Deno.stdout.write(encoder.encode("\x1b[C"));
              }
              i += 2;
              continue;
            }

            // Left arrow - move cursor left
            if (escapeCode === "D") {
              if (cursorPosition > 0) {
                cursorPosition--;
                await Deno.stdout.write(encoder.encode("\x1b[D"));
              }
              i += 2;
              continue;
            }

            // Home key (escape sequence varies)
            if (
              escapeCode === "H" ||
              (escapeCode === "1" &&
                i + 3 < input.length &&
                input[i + 3] === "~")
            ) {
              const charsToMove = cursorPosition;
              cursorPosition = 0;
              if (charsToMove > 0) {
                await Deno.stdout.write(encoder.encode(`\x1b[${charsToMove}D`));
              }
              i += escapeCode === "H" ? 2 : 3;
              continue;
            }

            // End key
            if (
              escapeCode === "F" ||
              (escapeCode === "4" &&
                i + 3 < input.length &&
                input[i + 3] === "~")
            ) {
              const charsToMove = currentLine.length - cursorPosition;
              cursorPosition = currentLine.length;
              if (charsToMove > 0) {
                await Deno.stdout.write(encoder.encode(`\x1b[${charsToMove}C`));
              }
              i += escapeCode === "F" ? 2 : 3;
              continue;
            }

            // Delete key
            if (
              escapeCode === "3" &&
              i + 3 < input.length &&
              input[i + 3] === "~"
            ) {
              if (cursorPosition < currentLine.length) {
                currentLine = currentLine.slice(0, cursorPosition) +
                  currentLine.slice(cursorPosition + 1);
                await redrawLine(prompt, currentLine, cursorPosition);
              }
              i += 3;
              continue;
            }

            i += 2;
            continue;
          }
        }

        // Handle backspace (127 or 8)
        if (code === 127 || code === 8) {
          if (cursorPosition > 0) {
            currentLine = currentLine.slice(0, cursorPosition - 1) +
              currentLine.slice(cursorPosition);
            cursorPosition--;
            await redrawLine(prompt, currentLine, cursorPosition);
          }
          continue;
        }

        // Handle Enter
        if (code === 13 || code === 10) {
          await Deno.stdout.write(encoder.encode("\n"));

          const trimmedLine = currentLine.trim();

          // Reset history navigation
          historyIndex = -1;

          // Add to history if non-empty
          if (trimmedLine.length > 0) {
            history.push(trimmedLine);
          }

          // Handle exit commands
          if (trimmedLine === "close()") {
            // Use logger if available in context
            if (
              "logger" in context &&
              typeof context.logger === "object" &&
              context.logger !== null &&
              "info" in context.logger
            ) {
              (context.logger as { info: (msg: string) => void }).info(
                "REPL session ended",
              );
            }
            return;
          }

          // Handle empty lines
          if (trimmedLine === "") {
            await Deno.stdout.write(encoder.encode(prompt));
            currentLine = "";
            cursorPosition = 0;
            continue;
          }

          // Handle help command
          if (trimmedLine === "help()") {
            console.log("Available commands:");
            console.log("  help()   - Show this help message");
            console.log("  close()  - Exit the REPL (or use Ctrl+C, Ctrl+D)");
            console.log("\nAvailable variables:");
            for (const key of Object.keys(context).sort()) {
              console.log(`  ${key}`);
            }
            console.log("\nLine editing:");
            console.log("  Up/Down  - Navigate command history");
            console.log("  Left/Right - Move cursor");
            console.log("  Home/End - Jump to start/end of line");
            console.log("  Backspace/Delete - Remove characters");
            await Deno.stdout.write(encoder.encode(prompt));
            currentLine = "";
            cursorPosition = 0;
            continue;
          }

          // Evaluate the expression
          try {
            // Create an async function that has access to our variables
            const AsyncFunction = async function () {}
              .constructor as typeof Function;
            const paramNames = Object.keys(context);
            const paramValues = Object.values(context);

            const result = await AsyncFunction(
              ...paramNames,
              `return (${trimmedLine})`,
            )(...paramValues);

            // Print the result
            console.log(result);
          } catch (error) {
            // If it fails as an expression, try as a statement
            try {
              const AsyncFunction = async function () {}
                .constructor as typeof Function;
              const paramNames = Object.keys(context);
              const paramValues = Object.values(context);

              await AsyncFunction(...paramNames, trimmedLine)(...paramValues);
            } catch (stmtError) {
              console.error(
                `Error: ${
                  stmtError instanceof Error
                    ? stmtError.message
                    : String(stmtError)
                }`,
              );
            }
          }

          // Write next prompt
          await Deno.stdout.write(encoder.encode(prompt));
          currentLine = "";
          cursorPosition = 0;
          continue;
        }

        // Handle printable characters
        if (code >= 32 && code < 127) {
          // Insert character at cursor position
          currentLine = currentLine.slice(0, cursorPosition) +
            char +
            currentLine.slice(cursorPosition);
          cursorPosition++;
          await redrawLine(prompt, currentLine, cursorPosition);
        }
      }
    }
  } finally {
    // Restore normal terminal mode
    Deno.stdin.setRaw(false);
  }
}

/**
 * Clears the current line content (preserves the prompt).
 *
 * This function moves the cursor back to the beginning of the user input
 * (after the prompt) and clears from that position to the end of the line.
 * The prompt itself is not affected.
 *
 * @param lineLength - The total length of the current line (unused but kept for API consistency)
 * @param cursorPos - The current cursor position within the line
 *
 * @internal
 */
async function clearLineContent(lineLength: number, cursorPos: number) {
  const encoder = new TextEncoder();
  // Move cursor back to the beginning of the input (after prompt)
  if (cursorPos > 0) {
    await Deno.stdout.write(encoder.encode(`\x1b[${cursorPos}D`));
  }
  // Clear from cursor to end of line
  await Deno.stdout.write(encoder.encode("\x1b[K"));
}

/**
 * Redraws the entire line and positions the cursor.
 *
 * This function performs a complete redraw of the current input line by:
 * 1. Moving to the start of the line (carriage return)
 * 2. Redrawing the prompt
 * 3. Writing the entire line content
 * 4. Clearing any remaining characters from previous longer lines
 * 5. Repositioning the cursor to the correct position
 *
 * This is used when characters are inserted or deleted in the middle of a line.
 *
 * @param prompt - The prompt string to display
 * @param line - The current line content to draw
 * @param cursorPos - The desired cursor position after redrawing
 *
 * @internal
 */
async function redrawLine(prompt: string, line: string, cursorPos: number) {
  const encoder = new TextEncoder();
  // Move to start of line (after prompt)
  await Deno.stdout.write(encoder.encode("\r" + prompt));
  // Write the line
  await Deno.stdout.write(encoder.encode(line));
  // Clear any remaining characters
  await Deno.stdout.write(encoder.encode("\x1b[K"));
  // Move cursor to correct position
  const charsFromEnd = line.length - cursorPos;
  if (charsFromEnd > 0) {
    await Deno.stdout.write(encoder.encode(`\x1b[${charsFromEnd}D`));
  }
}
