<template>
  <Modal
    ref="modalRef"
    :title="`Claude CLI - ${sessionDetails?.sessionId || 'Connecting...'}`"
    size="4xl"
    noEscClose
    @close="handleClose"
  >
    <div class="flex flex-col h-[80vh] bg-black text-green-400 font-mono text-sm">
      <!-- Terminal Header -->
      <div class="flex flex-row justify-between items-center px-3 py-2 bg-gray-800 border-b border-gray-600">
        <div class="flex items-center space-x-2">
          <div class="w-3 h-3 rounded-full bg-red-500"></div>
          <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
          <div class="w-3 h-3 rounded-full bg-green-500"></div>
          <span class="text-gray-300 text-xs ml-3">
            {{ sessionDetails?.status === RemoteShellStatus.Active ? 'Connected' : 'Connecting...' }}
          </span>
        </div>
        <div class="text-gray-300 text-xs">
          {{ sessionDetails?.containerId || 'N/A' }}
        </div>
      </div>

      <!-- Terminal Output Area -->
      <div
        ref="terminalOutputRef"
        class="flex-1 overflow-y-auto p-3 bg-black cursor-text"
        tabindex="0"
        @click="focusTerminal"
        @keydown="handleTerminalKeyDown"
      >
        <!-- Render terminal content with interactive elements -->
        <div ref="terminalContentRef" class="terminal-content"></div>
        
        <!-- Fallback input line if no active input detected -->
        <div v-if="!hasActiveInput" class="flex whitespace-pre-wrap break-words text-blue-300">
          <span class="text-blue-300 mr-1">🤖</span>
          <span>{{ currentInput }}</span>
          <span v-if="terminalFocused" class="bg-blue-300 text-black animate-pulse">█</span>
        </div>
      </div>



      <!-- Connection Status -->
      <div class="px-3 py-1 bg-gray-900 text-xs text-gray-400 border-t border-gray-700">
        Status: {{ connectionStatus }} | 
        Mode: Claude CLI |
        Session: {{ sessionDetails?.sessionId || 'N/A' }} |
        Container: {{ sessionDetails?.containerId || 'N/A' }}
      </div>
    </div>

    <!-- Loading overlay -->
    <div
      v-if="isConnecting"
      class="absolute inset-0 bg-black bg-opacity-75 flex items-center justify-center"
    >
      <div class="text-center text-white">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4"></div>
        <p>Starting Claude CLI...</p>
        <p class="text-sm text-gray-300 mt-2">{{ connectingMessage }}</p>
      </div>
    </div>

    <!-- Error overlay -->
    <div
      v-if="errorMessage"
      class="absolute inset-0 bg-red-900 bg-opacity-90 flex items-center justify-center"
    >
      <div class="text-center text-white">
        <div class="text-red-300 mb-4">⚠️ Connection Error</div>
        <p class="text-white">{{ errorMessage }}</p>
        <button
          class="mt-4 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
          @click="retry"
        >
          Retry Connection
        </button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { Modal } from "@si/vue-lib/design-system";
import { RemoteShellApi, CreateRemoteShellSessionResponse, RemoteShellStatus } from "@/api/sdf/dal/remote_shell";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
// No longer using NATS client - using WebSocket directly

interface TerminalLine {
  content: string;
  type: 'stdout' | 'stderr' | 'system' | 'input';
  timestamp: Date;
}

const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const terminalOutputRef = ref<HTMLElement>();
const terminalContentRef = ref<HTMLElement>();
const terminalInputRef = ref<HTMLInputElement>();
const interactiveInputRef = ref<HTMLInputElement>();

// Terminal state
const terminalLines = ref<TerminalLine[]>([]);
const currentInput = ref("");
const inputFocused = ref(false);
const terminalFocused = ref(true); // Start focused on terminal
const hasActiveInput = ref(false); // Track if Claude has an active input field
const commandHistory = ref<string[]>([]);
const historyIndex = ref(-1);

// Connection state  
const isConnecting = ref(false);
const connectingMessage = ref("");
const errorMessage = ref("");
const sessionDetails = ref<CreateRemoteShellSessionResponse | null>(null);
const connectionStatus = ref("Disconnected");
const isInteractiveMode = ref(true); // Always in Claude CLI mode

// WebSocket connection for shell I/O
const shellWebSocket = ref<WebSocket | null>(null);

const currentWorkspaceId = computed(() => workspacesStore.selectedWorkspacePk);
const currentChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);

// Add system message to terminal
function addSystemMessage(message: string) {
  if (terminalContentRef.value) {
    const systemHtml = `<div class="terminal-line whitespace-pre-wrap break-words text-blue-400">[System] ${message}</div>`;
    terminalContentRef.value.insertAdjacentHTML('beforeend', systemHtml);
  }
  scrollToBottom();
}

// Virtual cursor position for ANSI processing
let virtualCursorRow = 0;
let virtualCursorCol = 0;

// Process ANSI escape sequences and handle cursor movements properly
function processAnsiSequences(text: string): { content: string, shouldClear: boolean, shouldUpdateLine: boolean, targetLine: number } {
  // Convert back from escaped format
  const unescaped = text.replace(/\\u001b/g, '\u001b');
  
  const escapeChar = '\u001b';
  let processed = unescaped;
  let shouldClear = false;
  let shouldUpdateLine = false;
  let targetLine = -1;
  
  // Handle cursor movements and screen operations
  processed = processed.replace(new RegExp(`${escapeChar}\\[2J`, 'g'), () => {
    shouldClear = true;
    virtualCursorRow = 0;
    virtualCursorCol = 0;
    return '';
  });
  
  processed = processed.replace(new RegExp(`${escapeChar}\\[H`, 'g'), () => {
    virtualCursorRow = 0;
    virtualCursorCol = 0;
    shouldUpdateLine = true;
    targetLine = 0;
    return '';
  });
  
  // Handle cursor up movements
  processed = processed.replace(new RegExp(`${escapeChar}\\[([0-9]*)A`, 'g'), (match, num) => {
    const lines = parseInt(num) || 1;
    virtualCursorRow = Math.max(0, virtualCursorRow - lines);
    shouldUpdateLine = true;
    targetLine = virtualCursorRow;
    return '';
  });
  
  // Handle cursor position setting
  processed = processed.replace(new RegExp(`${escapeChar}\\[([0-9]*);([0-9]*)H`, 'g'), (match, row, col) => {
    virtualCursorRow = (parseInt(row) || 1) - 1;
    virtualCursorCol = (parseInt(col) || 1) - 1;
    shouldUpdateLine = true;
    targetLine = virtualCursorRow;
    return '';
  });
  
  // Handle line clearing
  processed = processed.replace(new RegExp(`${escapeChar}\\[K`, 'g'), () => {
    shouldUpdateLine = true;
    targetLine = virtualCursorRow;
    return '';
  });
  
  // Handle carriage returns (move to beginning of line)
  processed = processed.replace(/\r(?!\n)/g, () => {
    virtualCursorCol = 0;
    shouldUpdateLine = true;
    targetLine = virtualCursorRow;
    return '';
  });
  
  // Remove color codes
  processed = processed.replace(new RegExp(`${escapeChar}\\[[0-9;]*m`, 'g'), '');
  
  // Normalize line endings
  processed = processed.replace(/\r\n/g, '\n');
  
  return {
    content: processed,
    shouldClear,
    shouldUpdateLine,
    targetLine
  };
}

// Add output message to terminal with interactive HTML rendering
function addOutputMessage(message: string, type: 'stdout' | 'stderr' = 'stdout') {
  const processed = processAnsiSequences(message);
  
  // Handle clear screen
  if (processed.shouldClear) {
    if (terminalContentRef.value) {
      terminalContentRef.value.innerHTML = '';
    }
    virtualCursorRow = 0;
    virtualCursorCol = 0;
    hasActiveInput.value = false;
    return;
  }
  
  // Convert the text content to interactive HTML
  const htmlContent = convertToInteractiveHTML(processed.content, type);
  
  if (terminalContentRef.value && htmlContent.trim()) {
    // Handle line updates/replacements
    if (processed.shouldUpdateLine && processed.targetLine >= 0) {
      const lines = terminalContentRef.value.children;
      if (lines[processed.targetLine]) {
        (lines[processed.targetLine] as HTMLElement).outerHTML = htmlContent;
      } else {
        // Ensure we have enough lines
        while (lines.length <= processed.targetLine) {
          const emptyDiv = document.createElement('div');
          emptyDiv.className = 'terminal-line';
          terminalContentRef.value.appendChild(emptyDiv);
        }
        if (lines[processed.targetLine]) {
          (lines[processed.targetLine] as HTMLElement).outerHTML = htmlContent;
        }
      }
    } else {
      // Normal append
      terminalContentRef.value.insertAdjacentHTML('beforeend', htmlContent);
    }
    
    // Check for interactive elements
    detectInteractiveElements();
  }
  
  scrollToBottom();
}

// Convert terminal output to interactive HTML
function convertToInteractiveHTML(content: string, type: 'stdout' | 'stderr' | 'system' | 'input') {
  // Detect Claude's interactive patterns
  let html = content;
  
  // Check for input prompts - patterns like "Try: " or "> " that Claude uses for input
  if (content.includes('Try "') || content.includes('Try: ') || 
      content.includes('> ') || content.includes('What would you like to do?') ||
      content.includes('Enter your') || content.includes('Type your')) {
    
    // This looks like Claude is asking for input - create an interactive input field
    html = html.replace(/(Try ["']([^"']+)["']|Try: ([^\\n]+)|> |What would you like to do\?|Enter your[^\\n]*|Type your[^\\n]*)/g, 
      (match, fullMatch, quoted, unquoted) => {
        const placeholder = quoted || unquoted || 'Type here...';
        return `${match}<input type="text" class="claude-input bg-transparent border-b border-blue-300 text-blue-300 outline-none ml-2 flex-1" 
                placeholder="${placeholder}" 
                onkeydown="handleClaudeInput(event)" 
                onfocus="setActiveInput(true)" 
                onblur="setActiveInput(false)" />`;
      });
    hasActiveInput.value = true;
  }
  
  // Handle multi-line text areas if Claude creates them
  if (content.includes('```') || content.includes('Enter multiple lines')) {
    html = html.replace(/(Enter multiple lines[^\\n]*)/g, 
      `$1<textarea class="claude-textarea bg-black border border-blue-300 text-blue-300 outline-none mt-2 p-2 w-full min-h-[100px] font-mono" 
       placeholder="Type your multi-line input here..." 
       onkeydown="handleClaudeTextarea(event)"
       onfocus="setActiveInput(true)" 
       onblur="setActiveInput(false)"></textarea>`);
    hasActiveInput.value = true;
  }
  
  // Apply styling based on type
  let colorClass = 'text-green-400';
  if (type === 'stderr') {
    colorClass = 'text-red-400';
  } else if (type === 'system') {
    colorClass = 'text-blue-400';
  } else if (type === 'input') {
    colorClass = 'text-gray-300';
  }
  
  return `<div class="terminal-line whitespace-pre-wrap break-words ${colorClass}">${html}</div>`;
}

// Detect if there are interactive elements in the current terminal content
function detectInteractiveElements() {
  if (terminalContentRef.value) {
    const inputs = terminalContentRef.value.querySelectorAll('.claude-input, .claude-textarea');
    hasActiveInput.value = inputs.length > 0;
    
    // Auto-focus the last input element
    if (inputs.length > 0) {
      const lastInput = inputs[inputs.length - 1] as HTMLInputElement;
      nextTick(() => {
        lastInput.focus();
      });
    }
  }
}

// Add input message to terminal
function addInputMessage(command: string) {
  if (terminalContentRef.value) {
    const inputHtml = `<div class="terminal-line whitespace-pre-wrap break-words text-gray-300">$ ${command}</div>`;
    terminalContentRef.value.insertAdjacentHTML('beforeend', inputHtml);
  }
  scrollToBottom();
}

// Scroll terminal to bottom
async function scrollToBottom() {
  await nextTick();
  if (terminalOutputRef.value) {
    terminalOutputRef.value.scrollTop = terminalOutputRef.value.scrollHeight;
  }
}

// Focus the terminal area for direct typing
function focusTerminal() {
  terminalFocused.value = true;
  terminalOutputRef.value?.focus();
}

// Handle keyboard input directly in terminal area
function handleTerminalKeyDown(event: KeyboardEvent) {
  terminalFocused.value = true;
  
  if (event.key === 'Enter') {
    executeCommand();
  } else if (event.key === 'Backspace') {
    event.preventDefault();
    currentInput.value = currentInput.value.slice(0, -1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    navigateHistory(-1);
  } else if (event.key === 'ArrowDown') {
    event.preventDefault();
    navigateHistory(1);
  } else if (event.key.length === 1) {
    // Regular character input
    event.preventDefault();
    currentInput.value += event.key;
  }
  
  // Always scroll to bottom when typing
  scrollToBottom();
}


// Handle keyboard input
function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    executeCommand();
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    navigateHistory(-1);
  } else if (event.key === 'ArrowDown') {
    event.preventDefault();
    navigateHistory(1);
  } else if (event.key === 'Tab') {
    event.preventDefault();
    // TODO: Implement tab completion
  }
}

// Navigate command history
function navigateHistory(direction: number) {
  if (commandHistory.value.length === 0) return;
  
  const newIndex = historyIndex.value + direction;
  if (newIndex >= -1 && newIndex < commandHistory.value.length) {
    historyIndex.value = newIndex;
    if (newIndex === -1) {
      currentInput.value = '';
    } else {
      const command = commandHistory.value[commandHistory.value.length - 1 - newIndex];
      currentInput.value = command || '';
    }
  }
}

// Execute command
async function executeCommand() {
  const command = currentInput.value.trim();
  if (!command) return;

  // Add to history and display
  commandHistory.value.push(command);
  addInputMessage(command);
  currentInput.value = '';
  historyIndex.value = -1;

  // Handle local commands
  if (command === 'clear') {
    if (terminalContentRef.value) {
      terminalContentRef.value.innerHTML = '';
    }
    hasActiveInput.value = false;
    return;
  }

  if (command === 'exit') {
    handleClose();
    return;
  }

  // Send command to Claude CLI
  if (sessionDetails.value && sessionDetails.value.status === RemoteShellStatus.Active) {
    sendCommandToRemoteShell(command);
  } else {
    addOutputMessage("Error: Not connected to Claude CLI", 'stderr');
  }
}

// Send command to Claude CLI via WebSocket
function sendCommandToRemoteShell(command: string) {
  if (!shellWebSocket.value || shellWebSocket.value.readyState !== WebSocket.OPEN) {
    addOutputMessage("Error: Not connected to Claude CLI", 'stderr');
    return;
  }

  const message = JSON.stringify({ command });
  shellWebSocket.value.send(message);
}

// Create remote shell session
async function createSession() {
  if (!currentWorkspaceId.value || !currentChangeSetId.value) {
    errorMessage.value = "No workspace or change set selected";
    return;
  }

  try {
    isConnecting.value = true;
    connectingMessage.value = "Creating Claude CLI session...";
    errorMessage.value = "";

    const response = await RemoteShellApi.createSession(
      currentWorkspaceId.value,
      currentChangeSetId.value,
      {
        image: "ubuntu:20.04",
        workingDir: "/workspace",
        envVars: {
          USER: "si",
          HOME: "/workspace",
          PATH: "/usr/local/bin:/usr/bin:/bin"
        }
      }
    );

    sessionDetails.value = response.data;
    connectionStatus.value = response.data.status;
    
    addSystemMessage("Claude CLI session created successfully!");
    addSystemMessage(`Session ID: ${response.data.sessionId}`);
    addSystemMessage(`Container ID: ${response.data.containerId}`);
    
    if (response.data.message) {
      addSystemMessage(response.data.message);
    }

    // TODO: Connect to NATS subjects for shell I/O
    connectingMessage.value = "Connecting to Claude CLI...";
    await connectToNatsSubjects();

  } catch (error: any) {
    console.error("Failed to create remote shell session:", error);
    errorMessage.value = error.response?.data?.message || error.message || "Failed to create shell session";
  } finally {
    isConnecting.value = false;
    connectingMessage.value = "";
  }
}

// Connect to shell I/O via WebSocket (SDF handles NATS)
async function connectToNatsSubjects() {
  if (!sessionDetails.value) return;

  try {
    connectionStatus.value = "Connecting to Claude CLI...";
    
    // Connect to SDF WebSocket endpoint that relays shell I/O
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/api/v2/workspaces/${currentWorkspaceId.value}/change-sets/${currentChangeSetId.value}/remote-shell/connect/${sessionDetails.value.sessionId}`;
    
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log('Claude CLI WebSocket connected');
      connectionStatus.value = "Connected";
      addSystemMessage("Claude CLI connected successfully!");
      addSystemMessage("Type your messages and press Enter to chat with Claude.");
      
      // Focus terminal after connection
      nextTick(() => {
        focusTerminal();
      });
    };
    
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.type && message.content) {
          // Focus terminal when Claude starts or restarts
          if (message.type === 'system' && (message.content.includes('Started interactive command:') || message.content.includes('restarting'))) {
            // Focus the terminal area
            nextTick(() => {
              focusTerminal();
            });
          }
          addOutputMessage(message.content, message.type);
        }
      } catch (err) {
        console.warn('Failed to parse WebSocket message:', event.data);
        addOutputMessage(event.data, 'stdout');
      }
    };
    
    ws.onclose = (event) => {
      console.log('Claude CLI WebSocket disconnected:', event.code, event.reason);
      connectionStatus.value = "Disconnected";
      addSystemMessage("Claude CLI connection closed");
    };
    
    ws.onerror = (error) => {
      console.error('Claude CLI WebSocket error:', error);
      connectionStatus.value = "Connection Failed";
      addSystemMessage("Claude CLI connection error");
    };
    
    // Store WebSocket for sending commands
    shellWebSocket.value = ws;
    
  } catch (error: any) {
    console.error("Failed to connect to shell WebSocket:", error);
    errorMessage.value = "Failed to connect to Claude CLI";
    connectionStatus.value = "Connection Failed";
  }
}

// Retry connection
async function retry() {
  errorMessage.value = "";
  await createSession();
}

// Track if we're already closing to prevent infinite recursion
const isClosing = ref(false);

// Handle modal close
function handleClose() {
  if (isClosing.value) return; // Prevent recursive calls
  
  isClosing.value = true;
  
  try {
    // Close WebSocket connection
    if (shellWebSocket.value) {
      shellWebSocket.value.close();
      shellWebSocket.value = null;
    }
    
    connectionStatus.value = "Disconnected";
    modalRef.value?.close();
  } finally {
    // Reset the flag after a delay to allow for proper cleanup
    setTimeout(() => {
      isClosing.value = false;
    }, 100);
  }
}

// Public methods
const open = async () => {
  modalRef.value?.open();
  terminalLines.value = [];
  await createSession();
};

// Global handlers for Claude's interactive elements
(window as any).handleClaudeInput = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    const input = event.target as HTMLInputElement;
    const command = input.value;
    if (command.trim()) {
      // Send the command to Claude
      sendCommandToRemoteShell(command);
      // Clear the input
      input.value = '';
    }
  }
};

(window as any).handleClaudeTextarea = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
    const textarea = event.target as HTMLTextAreaElement;
    const command = textarea.value;
    if (command.trim()) {
      // Send the command to Claude
      sendCommandToRemoteShell(command);
      // Clear the textarea
      textarea.value = '';
    }
  }
};

(window as any).setActiveInput = (active: boolean) => {
  hasActiveInput.value = active;
};

defineExpose({
  open,
  close: () => modalRef.value?.close()
});

// Auto-focus terminal when modal opens
watch(() => modalRef.value?.isOpen, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      focusTerminal();
    });
  }
});
</script>

<style scoped>
/* Custom scrollbar for terminal */
.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1a1a1a;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #6a6a6a;
}

/* Blinking cursor animation */
@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.animate-pulse {
  animation: blink 1s infinite;
}
</style>