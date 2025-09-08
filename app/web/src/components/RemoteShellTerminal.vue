<template>
  <Modal
    ref="modalRef"
    :title="`Remote Shell - ${sessionDetails?.sessionId || 'Connecting...'}`"
    size="2xl"
    noEscClose
    @close="handleClose"
  >
    <div class="flex flex-col h-[600px] bg-black text-green-400 font-mono text-sm">
      <!-- Terminal Header -->
      <div class="flex flex-row justify-between items-center px-3 py-2 bg-gray-800 border-b border-gray-600">
        <div class="flex items-center space-x-2">
          <div class="w-3 h-3 rounded-full bg-red-500"></div>
          <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
          <div class="w-3 h-3 rounded-full bg-green-500"></div>
          <span class="text-gray-300 text-xs ml-3">
            {{ sessionDetails?.status === 'Active' ? 'Connected' : 'Connecting...' }}
          </span>
        </div>
        <div class="text-gray-300 text-xs">
          {{ sessionDetails?.containerId || 'N/A' }}
        </div>
      </div>

      <!-- Terminal Output Area -->
      <div
        ref="terminalOutputRef"
        class="flex-1 overflow-y-auto p-3 bg-black"
        @click="focusInput"
      >
        <div
          v-for="(line, index) in terminalLines"
          :key="index"
          class="whitespace-pre-wrap break-words"
          :class="{
            'text-red-400': line.type === 'stderr',
            'text-green-400': line.type === 'stdout',
            'text-blue-400': line.type === 'system',
            'text-gray-300': line.type === 'input'
          }"
        >{{ line.content }}</div>
      </div>

      <!-- Terminal Input -->
      <div class="flex items-center px-3 py-2 bg-black border-t border-gray-600">
        <span class="text-green-400 mr-2">$</span>
        <input
          ref="terminalInputRef"
          v-model="currentInput"
          class="flex-1 bg-transparent text-green-400 outline-none"
          placeholder="Type commands here..."
          @keydown="handleKeyDown"
          @focus="inputFocused = true"
          @blur="inputFocused = false"
        />
        <div
          v-if="inputFocused"
          class="w-2 h-4 bg-green-400 animate-pulse ml-1"
        ></div>
      </div>

      <!-- Connection Status -->
      <div class="px-3 py-1 bg-gray-900 text-xs text-gray-400 border-t border-gray-700">
        Status: {{ connectionStatus }} | 
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
        <p>Connecting to remote shell...</p>
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
import { createNatsClient, MockNatsClient, NatsWebSocketClient } from "@/utils/natsClient";

interface TerminalLine {
  content: string;
  type: 'stdout' | 'stderr' | 'system' | 'input';
  timestamp: Date;
}

const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const terminalOutputRef = ref<HTMLElement>();
const terminalInputRef = ref<HTMLInputElement>();

// Terminal state
const terminalLines = ref<TerminalLine[]>([]);
const currentInput = ref("");
const inputFocused = ref(false);
const commandHistory = ref<string[]>([]);
const historyIndex = ref(-1);

// Connection state  
const isConnecting = ref(false);
const connectingMessage = ref("");
const errorMessage = ref("");
const sessionDetails = ref<CreateRemoteShellSessionResponse | null>(null);
const connectionStatus = ref("Disconnected");

// NATS client connection
const natsClient = ref<MockNatsClient | NatsWebSocketClient | null>(null);
const unsubscribeFunctions = ref<(() => void)[]>([]);

const currentWorkspaceId = computed(() => workspacesStore.selectedWorkspacePk);
const currentChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);

// Add system message to terminal
function addSystemMessage(message: string) {
  terminalLines.value.push({
    content: `[System] ${message}`,
    type: 'system',
    timestamp: new Date()
  });
  scrollToBottom();
}

// Add output message to terminal
function addOutputMessage(message: string, type: 'stdout' | 'stderr' = 'stdout') {
  terminalLines.value.push({
    content: message,
    type,
    timestamp: new Date()
  });
  scrollToBottom();
}

// Add input message to terminal
function addInputMessage(command: string) {
  terminalLines.value.push({
    content: `$ ${command}`,
    type: 'input', 
    timestamp: new Date()
  });
  scrollToBottom();
}

// Scroll terminal to bottom
async function scrollToBottom() {
  await nextTick();
  if (terminalOutputRef.value) {
    terminalOutputRef.value.scrollTop = terminalOutputRef.value.scrollHeight;
  }
}

// Focus the input field
function focusInput() {
  terminalInputRef.value?.focus();
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
    terminalLines.value = [];
    return;
  }

  if (command === 'exit') {
    handleClose();
    return;
  }

  // Send command to remote shell (placeholder)
  if (sessionDetails.value && sessionDetails.value.status === RemoteShellStatus.Active) {
    sendCommandToRemoteShell(command);
  } else {
    addOutputMessage("Error: Not connected to remote shell", 'stderr');
  }
}

// Send command to remote shell via NATS
function sendCommandToRemoteShell(command: string) {
  if (!sessionDetails.value || !natsClient.value) {
    addOutputMessage("Error: Not connected to remote shell", 'stderr');
    return;
  }

  const stdinSubject = sessionDetails.value.connectionInfo.stdinSubject;
  natsClient.value.publish(stdinSubject, `${command}\n`);
}

// Create remote shell session
async function createSession() {
  if (!currentWorkspaceId.value || !currentChangeSetId.value) {
    errorMessage.value = "No workspace or change set selected";
    return;
  }

  try {
    isConnecting.value = true;
    connectingMessage.value = "Creating remote shell session...";
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
    
    addSystemMessage("Remote shell session created successfully!");
    addSystemMessage(`Session ID: ${response.data.sessionId}`);
    addSystemMessage(`Container ID: ${response.data.containerId}`);
    
    if (response.data.message) {
      addSystemMessage(response.data.message);
    }

    // TODO: Connect to NATS subjects for shell I/O
    connectingMessage.value = "Connecting to shell I/O...";
    await connectToNatsSubjects();

  } catch (error: any) {
    console.error("Failed to create remote shell session:", error);
    errorMessage.value = error.response?.data?.message || error.message || "Failed to create shell session";
  } finally {
    isConnecting.value = false;
    connectingMessage.value = "";
  }
}

// Connect to NATS subjects for shell I/O
async function connectToNatsSubjects() {
  if (!sessionDetails.value) return;

  try {
    // Create NATS client (using mock for now - in production you'd use the real client)
    natsClient.value = createNatsClient(false); // Set to true for real NATS connection
    
    // Connect to NATS
    await natsClient.value.connect();
    
    // Subscribe to stdout
    const stdoutUnsub = natsClient.value.subscribe(
      sessionDetails.value.connectionInfo.stdoutSubject,
      (data: string) => {
        addOutputMessage(data, 'stdout');
      }
    );
    
    // Subscribe to stderr
    const stderrUnsub = natsClient.value.subscribe(
      sessionDetails.value.connectionInfo.stderrSubject,
      (data: string) => {
        addOutputMessage(data, 'stderr');
      }
    );
    
    // Store unsubscribe functions for cleanup
    unsubscribeFunctions.value = [stdoutUnsub, stderrUnsub];
    
    connectionStatus.value = "Connected";
    addSystemMessage("Shell I/O connected successfully!");
    addSystemMessage("Type 'clear' to clear screen, 'exit' to close terminal");
    addSystemMessage("Try commands like: ls, pwd, whoami, date, echo hello");
    
    // Focus input after connection
    await nextTick();
    focusInput();
    
  } catch (error: any) {
    console.error("Failed to connect to NATS:", error);
    errorMessage.value = "Failed to connect to shell I/O";
    connectionStatus.value = "Connection Failed";
  }
}

// Retry connection
async function retry() {
  errorMessage.value = "";
  await createSession();
}

// Handle modal close
function handleClose() {
  // Clean up NATS subscriptions
  unsubscribeFunctions.value.forEach(unsub => unsub());
  unsubscribeFunctions.value = [];
  
  // Close NATS client connection
  if (natsClient.value) {
    natsClient.value.disconnect();
    natsClient.value = null;
  }
  
  connectionStatus.value = "Disconnected";
  modalRef.value?.close();
}

// Public methods
const open = async () => {
  modalRef.value?.open();
  terminalLines.value = [];
  await createSession();
};

defineExpose({
  open,
  close: () => modalRef.value?.close()
});

// Auto-focus input when modal opens
watch(() => modalRef.value?.isOpen, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      focusInput();
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