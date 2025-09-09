<template>
  <Modal
    ref="modalRef"
    :title="`Remote Shell - ${sessionDetails?.sessionId || 'Connecting...'}`"
    size="4xl"
    noEscClose
    @close="handleClose"
  >
    <div class="flex flex-col h-[80vh] bg-black">
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

      <!-- Real Terminal using xterm.js -->
      <div
        ref="terminalContainer"
        class="flex-1 bg-black"
      ></div>

      <!-- Connection Status -->
      <div class="px-3 py-1 bg-gray-900 text-xs text-gray-400 border-t border-gray-700">
        Status: {{ connectionStatus }} | 
        Mode: Shell |
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
        <p>Starting remote shell...</p>
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
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { Modal } from "@si/vue-lib/design-system";
import { RemoteShellApi, CreateRemoteShellSessionResponse, RemoteShellStatus } from "@/api/sdf/dal/remote_shell";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";

// Import xterm.js
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';
import 'xterm/css/xterm.css';

const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const terminalContainer = ref<HTMLElement>();

// Connection state  
const isConnecting = ref(false);
const connectingMessage = ref("");
const errorMessage = ref("");
const sessionDetails = ref<CreateRemoteShellSessionResponse | null>(null);
const connectionStatus = ref("Disconnected");

// WebSocket connection for shell I/O
const shellWebSocket = ref<WebSocket | null>(null);

// XTerm terminal instance
let terminal: Terminal | null = null;
let fitAddon: FitAddon | null = null;

const currentWorkspaceId = computed(() => workspacesStore.selectedWorkspacePk);
const currentChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);

// Initialize xterm.js terminal
function initializeTerminal() {
  if (!terminalContainer.value) return;

  terminal = new Terminal({
    cursorBlink: true,
    cursorStyle: 'block',
    fontSize: 14,
    fontFamily: 'Monaco, Menlo, "DejaVu Sans Mono", "Lucida Console", monospace',
    theme: {
      background: '#000000',
      foreground: '#ffffff',
      cursor: '#ffffff',
      black: '#000000',
      red: '#ff5555',
      green: '#50fa7b',
      yellow: '#f1fa8c',
      blue: '#bd93f9',
      magenta: '#ff79c6',
      cyan: '#8be9fd',
      white: '#bfbfbf',
      brightBlack: '#4d4d4d',
      brightRed: '#ff6e67',
      brightGreen: '#5af78e',
      brightYellow: '#f4f99d',
      brightBlue: '#caa9fa',
      brightMagenta: '#ff92d0',
      brightCyan: '#9aedfe',
      brightWhite: '#e6e6e6'
    },
    cols: 120,
    rows: 30
  });

  // Add addons
  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.loadAddon(new WebLinksAddon());

  // Open terminal in container
  terminal.open(terminalContainer.value);
  
  // Fit terminal to container
  fitAddon.fit();

  // Handle user input
  terminal.onData((data) => {
    // Send raw input directly to WebSocket - each character/key as it's typed
    if (shellWebSocket.value && shellWebSocket.value.readyState === WebSocket.OPEN) {
      // Send raw input, not as a command object
      shellWebSocket.value.send(JSON.stringify({ input: data }));
    }
  });

  // Handle terminal resize
  const handleResize = () => {
    if (fitAddon && terminal) {
      fitAddon.fit();
      // Send new dimensions to backend
      if (shellWebSocket.value && shellWebSocket.value.readyState === WebSocket.OPEN) {
        shellWebSocket.value.send(JSON.stringify({ 
          type: 'resize', 
          cols: terminal.cols, 
          rows: terminal.rows 
        }));
      }
    }
  };
  
  window.addEventListener('resize', handleResize);
  
  // Send initial resize after connection
  terminal.onResize((dimensions) => {
    if (shellWebSocket.value && shellWebSocket.value.readyState === WebSocket.OPEN) {
      shellWebSocket.value.send(JSON.stringify({ 
        type: 'resize', 
        cols: dimensions.cols, 
        rows: dimensions.rows 
      }));
    }
  });
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
          PATH: "/usr/local/bin:/usr/bin:/bin",
          TERM: "xterm-256color",
          COLUMNS: "120",
          LINES: "30"
        }
      }
    );

    sessionDetails.value = response.data;
    connectionStatus.value = response.data.status;
    
    if (terminal) {
      terminal.writeln("\x1b[36m• Remote shell session created\x1b[0m");
    }
    
    if (response.data.message) {
      terminal?.writeln(`\x1b[36m• ${response.data.message}\x1b[0m`);
    }

    connectingMessage.value = "Connecting to remote shell...";
    await connectToRemoteShell();

  } catch (error: any) {
    console.error("Failed to create remote shell session:", error);
    errorMessage.value = error.response?.data?.message || error.message || "Failed to create shell session";
  } finally {
    isConnecting.value = false;
    connectingMessage.value = "";
  }
}

// Connect to remote shell via WebSocket
async function connectToRemoteShell() {
  if (!sessionDetails.value) return;

  try {
    connectionStatus.value = "Connecting to remote shell...";
    
    // Connect to SDF WebSocket endpoint that relays shell I/O
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/api/v2/workspaces/${currentWorkspaceId.value}/change-sets/${currentChangeSetId.value}/remote-shell/connect/${sessionDetails.value.sessionId}`;
    
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log('Remote shell WebSocket connected');
      connectionStatus.value = "Connected";
      if (terminal) {
        terminal.writeln("\x1b[36m• Connected to shell session\x1b[0m");
        
        // Send terminal dimensions immediately after connection
        ws.send(JSON.stringify({ 
          type: 'resize', 
          cols: terminal.cols, 
          rows: terminal.rows 
        }));
        
        terminal.focus();
      }
    };
    
    ws.onmessage = (event) => {
      console.log('WebSocket message received:', event.data);
      try {
        const message = JSON.parse(event.data);
        console.log('Parsed message:', message);
        
        if (message.type && message.content && terminal) {
          console.log('Processing message type:', message.type);
          // Write raw output directly to terminal - let xterm handle all ANSI sequences
          if (message.type === 'stdout' || message.type === 'stderr') {
            // Debug: Log what we're receiving
            console.log('Raw message content:', JSON.stringify(message.content));
            
            // serde_json handles all escaping properly - just write directly to terminal
            terminal.write(message.content);
            
            // Debug: Log what we wrote to terminal
            console.log('Wrote to terminal:', JSON.stringify(message.content));
          } else if (message.type === 'system') {
            console.log('System message:', message.content);
            terminal.writeln(`\x1b[36m[System] ${message.content}\x1b[0m`);
          }
        } else {
          console.log('Message missing type/content or terminal not ready:', { type: message.type, hasContent: !!message.content, hasTerminal: !!terminal });
        }
      } catch (err) {
        console.warn('Failed to parse WebSocket message:', err, event.data);
        if (terminal) {
          terminal.write(event.data);
        }
      }
    };
    
    ws.onclose = (event) => {
      console.log('Remote shell WebSocket disconnected:', event.code, event.reason);
      connectionStatus.value = "Disconnected";
      if (terminal) {
        terminal.writeln("\x1b[31m• Shell connection closed\x1b[0m");
      }
    };
    
    ws.onerror = (error) => {
      console.error('Remote shell WebSocket error:', error);
      connectionStatus.value = "Connection Failed";
      if (terminal) {
        terminal.writeln("\x1b[31m• Shell connection error\x1b[0m");
      }
    };
    
    // Store WebSocket for sending commands
    shellWebSocket.value = ws;
    
  } catch (error: any) {
    console.error("Failed to connect to remote shell WebSocket:", error);
    errorMessage.value = "Failed to connect to remote shell";
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
  if (isClosing.value) return;
  
  isClosing.value = true;
  
  try {
    // Close WebSocket connection
    if (shellWebSocket.value) {
      shellWebSocket.value.close();
      shellWebSocket.value = null;
    }
    
    // Dispose terminal
    if (terminal) {
      terminal.dispose();
      terminal = null;
    }
    
    connectionStatus.value = "Disconnected";
    modalRef.value?.close();
  } finally {
    setTimeout(() => {
      isClosing.value = false;
    }, 100);
  }
}

// Public methods
const open = async () => {
  modalRef.value?.open();
  
  // Initialize terminal after modal is open
  await nextTick();
  initializeTerminal();
  
  // Start session
  await createSession();
};

defineExpose({
  open,
  close: () => modalRef.value?.close()
});

// Cleanup on unmount
onUnmounted(() => {
  if (terminal) {
    terminal.dispose();
  }
  if (shellWebSocket.value) {
    shellWebSocket.value.close();
  }
});
</script>

<style scoped>
/* Terminal container should fill available space */
:deep(.xterm) {
  height: 100%;
}

:deep(.xterm-viewport) {
  background-color: #000000 !important;
}
</style>