// Simple NATS WebSocket client for browser
// This is a basic implementation for connecting to NATS subjects via WebSocket

export class NatsWebSocketClient {
  private ws: WebSocket | null = null;
  private subscribers: Map<string, (data: string) => void> = new Map();
  private connected = false;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor(private url: string) {}

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          console.log('NATS WebSocket connected');
          this.connected = true;
          this.reconnectAttempts = 0;
          resolve();
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        this.ws.onclose = (event) => {
          console.log('NATS WebSocket disconnected:', event.code, event.reason);
          this.connected = false;
          this.handleReconnect();
        };

        this.ws.onerror = (error) => {
          console.error('NATS WebSocket error:', error);
          if (!this.connected) {
            reject(error);
          }
        };

      } catch (error) {
        reject(error);
      }
    });
  }

  private handleMessage(data: string) {
    try {
      // Parse NATS message format
      // For now, we'll assume messages are in the format: "subject:payload"
      const [subject, ...payloadParts] = data.split(':');
      if (!subject) return;
      
      const payload = payloadParts.join(':');
      const handler = this.subscribers.get(subject);
      if (handler) {
        handler(payload);
      }
    } catch (error) {
      console.error('Failed to handle NATS message:', error);
    }
  }

  private async handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      
      setTimeout(() => {
        this.connect().catch(error => {
          console.error('Reconnection failed:', error);
        });
      }, this.reconnectDelay * this.reconnectAttempts);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  subscribe(subject: string, handler: (data: string) => void): () => void {
    this.subscribers.set(subject, handler);

    // Send subscription message to NATS (if protocol supported this)
    if (this.connected && this.ws) {
      const subscribeMsg = JSON.stringify({
        type: 'subscribe',
        subject
      });
      this.ws.send(subscribeMsg);
    }

    // Return unsubscribe function
    return () => {
      this.subscribers.delete(subject);
      if (this.connected && this.ws) {
        const unsubscribeMsg = JSON.stringify({
          type: 'unsubscribe', 
          subject
        });
        this.ws.send(unsubscribeMsg);
      }
    };
  }

  publish(subject: string, data: string): void {
    if (this.connected && this.ws) {
      const message = JSON.stringify({
        type: 'publish',
        subject,
        data
      });
      this.ws.send(message);
    } else {
      console.warn('Cannot publish - NATS WebSocket not connected');
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.connected = false;
    this.subscribers.clear();
  }

  isConnected(): boolean {
    return this.connected;
  }
}

// Create a simple mock NATS client for development/testing
export class MockNatsClient {
  private subscribers: Map<string, (data: string) => void> = new Map();

  // eslint-disable-next-line class-methods-use-this
  async connect(): Promise<void> {
    console.log('Mock NATS client connected');
  }

  subscribe(subject: string, handler: (data: string) => void): () => void {
    this.subscribers.set(subject, handler);
    console.log(`Subscribed to mock subject: ${subject}`);

    // Return unsubscribe function
    return () => {
      this.subscribers.delete(subject);
      console.log(`Unsubscribed from mock subject: ${subject}`);
    };
  }

  publish(subject: string, data: string): void {
    console.log(`Mock publish to ${subject}:`, data);
    
    // Simulate command execution for demo
    if (subject.endsWith('.stdin')) {
      // Simulate shell response after a short delay
      setTimeout(() => {
        const outputSubject = subject.replace('.stdin', '.stdout');
        const handler = this.subscribers.get(outputSubject);
        if (handler) {
          // Mock shell output
          const mockOutput = MockNatsClient.generateMockOutput(data);
          handler(mockOutput);
        }
      }, 200);
    }
  }

  private static generateMockOutput(command: string): string {
    const cmd = command.trim();
    
    // Mock different command outputs
    if (cmd === 'ls') {
      return 'bin  etc  home  lib  usr  var';
    } else if (cmd === 'pwd') {
      return '/workspace';
    } else if (cmd.startsWith('echo ')) {
      return cmd.substring(5);
    } else if (cmd === 'whoami') {
      return 'si';
    } else if (cmd === 'date') {
      return new Date().toString();
    } else if (cmd === 'ps') {
      return '  PID TTY          TIME CMD\n    1 ?        00:00:01 bash\n   42 ?        00:00:00 ps';
    } else {
      return `${cmd}: command not found (mock shell)`;
    }
  }

  disconnect(): void {
    this.subscribers.clear();
    console.log('Mock NATS client disconnected');
  }

  // eslint-disable-next-line class-methods-use-this
  isConnected(): boolean {
    return true;
  }
}

// Factory function to create appropriate client
export function createNatsClient(useReal = false): NatsWebSocketClient | MockNatsClient {
  if (useReal) {
    // In a real implementation, this would connect to the actual NATS WebSocket endpoint
    // The URL would need to be configured based on your NATS setup
    const natsWsUrl = 'ws://localhost:4222'; // This would need to be your actual NATS WebSocket URL
    return new NatsWebSocketClient(natsWsUrl);
  } else {
    return new MockNatsClient();
  }
}