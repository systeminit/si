<template>
  <div>
    <!-- Terminal Panel States -->
    
    <!-- Navbar shell icon: Always visible with different states -->
  <div 
    v-tooltip="{
      content: 'Remote Terminal',
    }"
    @click="handleNavbarIconClick"
    class="flex items-center justify-center w-8 h-8 bg-gray-800 rounded transition-all relative"
    :class="{ 
      'ring-2 ring-green-500': connectionStatus === 'Connected',
      'opacity-50 cursor-pointer hover:opacity-75': panelState === 'sidebar' || panelState === 'popped-out',
      'opacity-100 cursor-pointer hover:bg-gray-700': panelState === 'minimized'
    }"
  >
    <Icon name="command" class="w-4 h-4 text-gray-300" />
    <div 
      v-if="connectionStatus === 'Connected'"
      class="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full"
    ></div>
  </div>

  <!-- Sidebar panel state -->
  <div 
    v-show="panelState === 'sidebar'"
    class="fixed right-0 top-0 h-full bg-gray-900 shadow-2xl z-50 flex flex-col border-l border-gray-700"
    :style="{ width: sidebarWidth + 'px' }"
  >
    <!-- Resize handle -->
    <div 
      ref="resizeHandle"
      class="absolute left-0 top-1/2 transform -translate-y-1/2 w-2 h-20 bg-gray-600 hover:bg-gray-400 cursor-ew-resize rounded-r-md transition-colors z-10 flex items-center justify-center opacity-70 hover:opacity-100"
      @mousedown="startResize"
      title="Drag to resize terminal width"
    >
      <!-- Visual grip lines -->
      <div class="flex flex-col space-y-0.5">
        <div class="w-0.5 h-1 bg-gray-300 rounded"></div>
        <div class="w-0.5 h-1 bg-gray-300 rounded"></div>
        <div class="w-0.5 h-1 bg-gray-300 rounded"></div>
      </div>
    </div>
    
    <!-- Panel Header -->
    <div class="flex items-center justify-between p-3 bg-gray-800 border-b border-gray-700">
      <div class="flex items-center space-x-2">
        <Icon name="command" class="w-4 h-4 text-gray-300" />
        <span class="text-sm font-medium text-gray-300">Terminal</span>
        <div 
          class="w-2 h-2 rounded-full"
          :class="connectionStatus === 'Connected' ? 'bg-green-500' : 'bg-gray-500'"
        ></div>
      </div>
      <div class="flex space-x-1">
        <button 
          @click="popOut"
          class="p-1 hover:bg-gray-700 rounded text-gray-400 hover:text-gray-200"
          title="Pop out"
        >
          <Icon name="plus" class="w-4 h-4" />
        </button>
        <button 
          @click="minimizePanel"
          class="p-1 hover:bg-gray-700 rounded text-gray-400 hover:text-gray-200"
          title="Minimize"
        >
          <Icon name="minus" class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Terminal Container -->
    <div class="flex-1 bg-black relative">
      <div ref="terminalContainer" class="w-full h-full"></div>
      
      <!-- Loading overlay -->
      <div
        v-if="isConnecting"
        class="absolute inset-0 bg-black bg-opacity-75 flex items-center justify-center"
      >
        <div class="text-center text-white">
          <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-white mx-auto mb-2"></div>
          <p class="text-sm">{{ connectingMessage || 'Connecting...' }}</p>
        </div>
      </div>

      <!-- Error overlay -->
      <div
        v-if="errorMessage"
        class="absolute inset-0 bg-red-900 bg-opacity-90 flex items-center justify-center"
      >
        <div class="text-center text-white">
          <div class="text-red-300 mb-2">⚠️ Connection Error</div>
          <p class="text-sm text-white mb-3">{{ errorMessage }}</p>
          <button
            class="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700"
            @click="retry"
          >
            Retry
          </button>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div class="px-3 py-1 bg-gray-800 text-xs text-gray-400 border-t border-gray-700">
      <div class="flex justify-between">
        <span>{{ connectionStatus }}</span>
        <span v-if="sessionDetails">{{ sessionDetails.sessionId.substring(0, 8) }}</span>
      </div>
    </div>
  </div>

  <!-- Popped out overlay state - full screen -->
  <div 
    v-show="panelState === 'popped-out'"
    class="fixed inset-0 bg-gray-900 shadow-2xl z-50 flex flex-col"
  >
    <!-- Header -->
    <div 
      class="flex items-center justify-between p-3 bg-gray-800 border-b border-gray-700"
    >
      <div class="flex items-center space-x-2">
        <div class="flex space-x-1">
          <div class="w-3 h-3 rounded-full bg-red-500"></div>
          <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
          <div class="w-3 h-3 rounded-full bg-green-500"></div>
        </div>
        <span class="text-sm font-medium text-gray-300 ml-2">Terminal - {{ sessionDetails?.sessionId || 'Connecting...' }}</span>
        <div 
          class="w-2 h-2 rounded-full"
          :class="connectionStatus === 'Connected' ? 'bg-green-500' : 'bg-gray-500'"
        ></div>
      </div>
      <div class="flex space-x-1">
        <button 
          @click="popIn"
          class="p-1 hover:bg-gray-700 rounded text-gray-400 hover:text-gray-200"
          title="Return to sidebar"
        >
          <Icon name="chevron--right" class="w-4 h-4" />
        </button>
        <button 
          @click="minimizePanel"
          class="p-1 hover:bg-gray-700 rounded text-gray-400 hover:text-gray-200"
          title="Minimize"
        >
          <Icon name="minus" class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Terminal Container -->
    <div class="flex-1 bg-black relative min-h-0">
      <!-- Shared terminal container will be moved here dynamically -->
      
      <!-- Loading/Error overlays (same as sidebar) -->
      <div
        v-if="isConnecting"
        class="absolute inset-0 bg-black bg-opacity-75 flex items-center justify-center"
      >
        <div class="text-center text-white">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4"></div>
          <p>{{ connectingMessage || 'Connecting...' }}</p>
        </div>
      </div>

      <div
        v-if="errorMessage"
        class="absolute inset-0 bg-red-900 bg-opacity-90 flex items-center justify-center"
      >
        <div class="text-center text-white">
          <div class="text-red-300 mb-4">⚠️ Connection Error</div>
          <p class="text-white mb-4">{{ errorMessage }}</p>
          <button
            class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
            @click="retry"
          >
            Retry Connection
          </button>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div class="px-3 py-1 bg-gray-800 text-xs text-gray-400 border-t border-gray-700">
      <div class="flex justify-between">
        <span>{{ connectionStatus }} | Session: {{ sessionDetails?.sessionId || 'N/A' }}</span>
        <span>Container: {{ sessionDetails?.containerId || 'N/A' }}</span>
      </div>
    </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, reactive, watch } from "vue";
import { Icon } from "@si/vue-lib/design-system";
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

// Panel states: 'minimized' | 'sidebar' | 'popped-out'
const panelState = ref<'minimized' | 'sidebar' | 'popped-out'>('minimized');

// Single terminal container for all states
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

// Session persistence - keep trying to maintain connection
let sessionTimeout: number | null = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 3;

// Sidebar resizing functionality  
const sidebarWidth = ref(662); // Default 36rem + 15% = 576px * 1.15 = 662px
const resizeHandle = ref<HTMLElement>();
let isResizing = false;
const minWidth = 320; // Minimum 20rem
const maxWidth = 1200; // Maximum 75rem

const currentWorkspaceId = computed(() => workspacesStore.selectedWorkspacePk);
const currentChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);

// Initialize xterm.js terminal - only create once to prevent flicker
function initializeTerminal() {
  // If terminal already exists, just resize it
  if (terminal && fitAddon) {
    nextTick(() => {
      fitAddon?.fit();
    });
    return;
  }

  // Only create terminal if we don't have one yet
  const container = terminalContainer.value;
  if (!container) return;

  terminal = new Terminal({
    cursorBlink: true,
    cursorStyle: 'block',
    fontSize: 13,
    fontFamily: 'Monaco, Menlo, "DejaVu Sans Mono", "Lucida Console", monospace',
    rightClickSelectsWord: false, // Disable right click selection to avoid paste conflicts
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
    cols: panelState.value === 'popped-out' ? 160 : Math.floor(sidebarWidth.value / 9), // Approximate chars per pixel
    rows: panelState.value === 'popped-out' ? 50 : 35
  });

  // Add addons
  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);
  terminal.loadAddon(new WebLinksAddon());

  // Open terminal in container
  terminal.open(container);
  
  // Disable bracketed paste mode to fix paste issues
  terminal.write('\x1b[?2004l');
  
  // Fit terminal to container
  nextTick(() => {
    fitAddon?.fit();
  });

  // Handle user input
  terminal.onData((data) => {
    // Reset session timeout on activity
    resetSessionTimeout();
    
    if (shellWebSocket.value && shellWebSocket.value.readyState === WebSocket.OPEN) {
      shellWebSocket.value.send(JSON.stringify({ input: data }));
    }
  });

  // Handle terminal resize with debouncing to prevent flicker
  let resizeTimeout: number | null = null;
  const handleResize = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout);
    }
    
    resizeTimeout = window.setTimeout(() => {
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
      resizeTimeout = null;
    }, 100);
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

// Panel state management
function togglePanel() {
  if (panelState.value === 'minimized') {
    panelState.value = 'sidebar';
    nextTick(() => {
      initializeTerminal();
      // Only ensure connection if we don't have one or it's disconnected
      if (connectionStatus.value !== 'Connected') {
        ensureConnection();
      }
    });
  } else {
    minimizePanel();
  }
}

const handleNavbarIconClick = () => {
  if (panelState.value === 'minimized') {
    // Normal behavior - expand to sidebar
    togglePanel();
  } else if (panelState.value === 'sidebar') {
    // If sidebar is open, minimize it
    minimizePanel();
  } else if (panelState.value === 'popped-out') {
    // If popped out, minimize it (user probably wants to get back to normal view)
    minimizePanel();
  }
};

function minimizePanel() {
  panelState.value = 'minimized';
  // Keep connection alive and terminal instance - just hide the UI
  // Terminal content will be preserved when re-expanded
}

function popOut() {
  const oldState = panelState.value;
  panelState.value = 'popped-out';
  
  // Only reinitialize terminal if we're coming from minimized state
  if (oldState === 'minimized') {
    nextTick(() => {
      initializeTerminal();
    });
  } else {
    // Terminal container stays the same, just resize
    nextTick(() => {
      if (fitAddon && terminal) {
        fitAddon.fit();
      }
    });
  }
}

function popIn() {
  const oldState = panelState.value;
  panelState.value = 'sidebar';
  
  // Only reinitialize terminal if we're coming from minimized state  
  if (oldState === 'minimized') {
    nextTick(() => {
      initializeTerminal();
    });
  } else {
    // Terminal container stays the same, just resize
    nextTick(() => {
      if (fitAddon && terminal) {
        fitAddon.fit();
      }
    });
  }
}

// Session timeout management
function resetSessionTimeout() {
  if (sessionTimeout) {
    clearTimeout(sessionTimeout);
  }
  
  // Reset session timeout to 10 minutes
  sessionTimeout = window.setTimeout(() => {
    if (connectionStatus.value === 'Connected') {
      connectionStatus.value = 'Session timed out';
      closeConnection();
    }
  }, 10 * 60 * 1000);
}

// Connection management with persistence
async function ensureConnection() {
  if (!sessionDetails.value && connectionStatus.value !== 'Connected') {
    await createSession();
  } else if (connectionStatus.value === 'Connected') {
    // Re-initialize terminal output if we have an existing connection
    connectToExistingSession();
  }
}

async function createSession() {
  if (!currentWorkspaceId.value || !currentChangeSetId.value) {
    errorMessage.value = "No workspace or change set selected";
    return;
  }

  try {
    isConnecting.value = true;
    connectingMessage.value = "Creating remote shell session...";
    errorMessage.value = "";
    reconnectAttempts = 0;

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
          TERM: "xterm-256color"
        }
      }
    );

    sessionDetails.value = response.data;
    connectionStatus.value = response.data.status;
    
    connectingMessage.value = "Connecting to remote shell...";
    await connectToRemoteShell();
    
    // Start session timeout tracking
    resetSessionTimeout();

  } catch (error: any) {
    console.error("Failed to create remote shell session:", error);
    errorMessage.value = error.response?.data?.message || error.message || "Failed to create shell session";
  } finally {
    isConnecting.value = false;
    connectingMessage.value = "";
  }
}

async function connectToRemoteShell() {
  if (!sessionDetails.value) return;

  try {
    connectionStatus.value = "Connecting...";
    
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/api/v2/workspaces/${currentWorkspaceId.value}/change-sets/${currentChangeSetId.value}/remote-shell/connect/${sessionDetails.value.sessionId}`;
    
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log('Remote shell WebSocket connected');
      connectionStatus.value = "Connected";
      reconnectAttempts = 0;
      
      if (terminal) {
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
      try {
        const message = JSON.parse(event.data);
        
        if (message.type && message.content && terminal) {
          if (message.type === 'stdout' || message.type === 'stderr') {
            terminal.write(message.content);
          } else if (message.type === 'system') {
            terminal.writeln(`\x1b[36m[System] ${message.content}\x1b[0m`);
          }
        }
      } catch (err) {
        console.warn('Failed to parse WebSocket message:', err);
        if (terminal) {
          terminal.write(event.data);
        }
      }
    };
    
    ws.onclose = (event) => {
      console.log('Remote shell WebSocket disconnected:', event.code, event.reason);
      
      if (connectionStatus.value === 'Connected') {
        connectionStatus.value = "Disconnected";
        
        // Attempt to reconnect if not intentionally closed
        if (reconnectAttempts < maxReconnectAttempts && event.code !== 1000) {
          reconnectAttempts++;
          setTimeout(() => {
            console.log(`Attempting to reconnect (${reconnectAttempts}/${maxReconnectAttempts})`);
            connectToRemoteShell();
          }, 2000 * reconnectAttempts);
        }
      }
    };
    
    ws.onerror = (error) => {
      console.error('Remote shell WebSocket error:', error);
      connectionStatus.value = "Connection Failed";
    };
    
    shellWebSocket.value = ws;
    
  } catch (error: any) {
    console.error("Failed to connect to remote shell WebSocket:", error);
    errorMessage.value = "Failed to connect to remote shell";
    connectionStatus.value = "Connection Failed";
  }
}

function connectToExistingSession() {
  // If we have a session but no active WebSocket, reconnect
  if (sessionDetails.value && (!shellWebSocket.value || shellWebSocket.value.readyState !== WebSocket.OPEN)) {
    connectToRemoteShell();
  }
}

function closeConnection() {
  if (shellWebSocket.value) {
    shellWebSocket.value.close(1000, 'User closed');
    shellWebSocket.value = null;
  }
  
  if (sessionTimeout) {
    clearTimeout(sessionTimeout);
    sessionTimeout = null;
  }
}

async function retry() {
  errorMessage.value = "";
  closeConnection();
  sessionDetails.value = null;
  await createSession();
}

// Resize functionality for sidebar
function startResize(e: MouseEvent) {
  isResizing = true;
  document.addEventListener('mousemove', handleResize);
  document.addEventListener('mouseup', stopResize);
  e.preventDefault();
}

function handleResize(e: MouseEvent) {
  if (!isResizing) return;
  
  // Calculate new width based on distance from right edge of viewport
  const newWidth = window.innerWidth - e.clientX;
  
  // Constrain within min/max bounds
  sidebarWidth.value = Math.min(maxWidth, Math.max(minWidth, newWidth));
  
  // Re-fit terminal to new dimensions
  if (terminal && fitAddon) {
    nextTick(() => {
      fitAddon?.fit();
      // Send new dimensions to backend
      if (shellWebSocket.value && shellWebSocket.value.readyState === WebSocket.OPEN) {
        shellWebSocket.value.send(JSON.stringify({ 
          type: 'resize', 
          cols: terminal?.cols || 80, 
          rows: terminal?.rows || 30 
        }));
      }
    });
  }
}

function stopResize() {
  isResizing = false;
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
}


// Watch for workspace/changeset changes to maintain sessions
watch([currentWorkspaceId, currentChangeSetId], () => {
  if (panelState.value !== 'minimized') {
    // Workspace/changeset changed, need new session
    closeConnection();
    sessionDetails.value = null;
    ensureConnection();
  }
});

// Auto-connect when component mounts if there's an active workspace/changeset
onMounted(() => {
  if (currentWorkspaceId.value && currentChangeSetId.value) {
    // Don't auto-open, just prepare for when user opens
  }
});

// Cleanup on unmount
onUnmounted(() => {
  if (terminal) {
    terminal.dispose();
  }
  closeConnection();
  
  // Clean up resize event listeners
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
});

// Public API
defineExpose({
  open: togglePanel,
  close: minimizePanel,
  connectionStatus: () => connectionStatus.value,
  isConnected: () => connectionStatus.value === 'Connected'
});
</script>

<style scoped>
/* Terminal container should fill available space */
:deep(.xterm) {
  height: 100%;
  width: 100%;
  /* Prevent flicker during container changes */
  transition: none !important;
}

:deep(.xterm-viewport) {
  background-color: #000000 !important;
  /* Smooth rendering without jarring transitions */
  backface-visibility: hidden;
  transform: translateZ(0);
}


/* Smooth transitions */
.transition-colors {
  transition: background-color 0.15s ease;
}
</style>