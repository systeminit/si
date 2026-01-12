/**
 * Soak Test Utilities
 *
 * Framework for running performance and load tests against the Luminork API.
 * Provides progress tracking, metrics collection, and reporting similar to
 * standalone soak test scripts.
 */

export interface SoakTestConfig {
  /** Total number of operations to perform (or unlimited if 0) */
  iterations: number;
  /** Maximum test duration in milliseconds (or unlimited if 0) */
  duration?: number;
  /** Number of parallel test threads (each gets own changeset) */
  parallelThreads?: number;
  /** Number of concurrent operations within each thread (default: 1) */
  parallelism?: number;
  /** Whether to cleanup resources after test completion (default: true) */
  cleanup?: boolean;
  /** Report progress every N operations (default: 100) */
  reportInterval?: number;
}

export interface SoakMetrics {
  /** Total number of operations attempted across all threads */
  totalOperations: number;
  /** Number of successful operations across all threads */
  successfulOperations: number;
  /** Number of failed operations across all threads */
  failedOperations: number;
  /** Average time per operation in milliseconds */
  averageOperationTimeMs: number;
  /** Operations per second across all threads */
  operationsPerSecond: number;
  /** Total test duration in milliseconds */
  totalDurationMs: number;
  /** Test start time */
  startTime: Date;
  /** Test end time */
  endTime: Date;
  /** Collection of errors encountered */
  errors: Array<{ thread: number; operation: number; error: string; timestamp: Date }>;
  /** Per-thread metrics */
  threadMetrics: SoakThreadMetrics[];
}

export interface SoakThreadMetrics {
  /** Thread identifier */
  threadId: number;
  /** Changeset ID used by this thread */
  changeSetId: string;
  /** Operations completed by this thread */
  operations: number;
  /** Successful operations by this thread */
  successful: number;
  /** Failed operations by this thread */
  failed: number;
  /** Thread execution time in milliseconds */
  durationMs: number;
}

export interface SoakOperationResult {
  success: boolean;
  durationMs: number;
  error?: string;
  data?: unknown;
}

export type SoakOperation<T> = (iteration: number, changeSetId: string) => Promise<T>;
export type SoakChangeSetFactory = (threadId: number) => Promise<string>;

/**
 * Soak Test Runner
 * 
 * Executes parallel threads of operations with duration and iteration limits.
 * Each thread gets its own changeset and runs operations until either:
 * 1. Maximum iterations reached, OR 2. Maximum duration reached
 */
export class SoakTestRunner {
  private config: Required<SoakTestConfig>;

  constructor(config: SoakTestConfig) {
    this.config = {
      parallelThreads: 1,
      parallelism: 1,
      duration: 0, // unlimited by default
      cleanup: true,
      reportInterval: 100,
      ...config,
    };
  }

  /**
   * Run a soak test with parallel threads, each using its own changeset
   */
  async run<T>(
    changeSetFactory: SoakChangeSetFactory,
    operation: SoakOperation<T>
  ): Promise<SoakMetrics> {
    const startTime = new Date();
    const allErrors: Array<{ thread: number; operation: number; error: string; timestamp: Date }> = [];
    const threadMetrics: SoakThreadMetrics[] = [];

    console.log('============================================================');
    console.log(`🚀 Starting soak test with ${this.config.parallelThreads} parallel threads`);
    console.log(`⏱️  Max duration: ${this.config.duration ? (this.config.duration / 1000) + 's' : 'unlimited'}`);
    console.log(`🔄 Max iterations per thread: ${this.config.iterations || 'unlimited'}`);

    // Create and start all threads
    const threadPromises: Promise<SoakThreadMetrics>[] = [];
    
    for (let threadId = 1; threadId <= this.config.parallelThreads; threadId++) {
      const threadPromise = this.runThread(threadId, changeSetFactory, operation, startTime);
      threadPromises.push(threadPromise);
    }

    try {
      // Wait for all threads to complete
      const results = await Promise.all(threadPromises);
      threadMetrics.push(...results);

      // Aggregate results from all threads
      const endTime = new Date();
      const totalDurationMs = endTime.getTime() - startTime.getTime();
      
      const totalOperations = threadMetrics.reduce((sum, tm) => sum + tm.operations, 0);
      const successfulOperations = threadMetrics.reduce((sum, tm) => sum + tm.successful, 0);
      const failedOperations = threadMetrics.reduce((sum, tm) => sum + tm.failed, 0);

      // Calculate average operation time across all threads
      const allThreadDurations = threadMetrics.map(tm => tm.durationMs);
      const averageOperationTimeMs = allThreadDurations.length > 0
        ? allThreadDurations.reduce((sum, duration) => sum + duration, 0) / totalOperations
        : 0;

      const operationsPerSecond = totalDurationMs > 0 
        ? (successfulOperations / totalDurationMs) * 1000 
        : 0;

      const metrics: SoakMetrics = {
        totalOperations,
        successfulOperations,
        failedOperations,
        averageOperationTimeMs,
        operationsPerSecond,
        totalDurationMs,
        startTime,
        endTime,
        errors: allErrors,
        threadMetrics,
      };

      this.reportFinalResults(metrics);
      return metrics;

    } catch (error) {
      console.error(`❌ Soak test failed: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  /**
   * Run a single thread with its own changeset
   */
  private async runThread<T>(
    threadId: number,
    changeSetFactory: SoakChangeSetFactory,
    operation: SoakOperation<T>,
    globalStartTime: Date
  ): Promise<SoakThreadMetrics> {
    const threadStartTime = Date.now();
    let operations = 0;
    let successful = 0;
    let failed = 0;

    // Create changeset for this thread
    const changeSetId = await changeSetFactory(threadId);
    console.log(`📝 Thread ${threadId}: Using changeset ${changeSetId}`);

    // Create duration timeout if specified
    const hasTimeLimit = this.config.duration > 0;
    const timeLimit = hasTimeLimit ? globalStartTime.getTime() + this.config.duration : Number.MAX_SAFE_INTEGER;

    // Run operations until iteration limit OR time limit reached
    while (
      (this.config.iterations === 0 || operations < this.config.iterations) &&
      (!hasTimeLimit || Date.now() < timeLimit)
    ) {
      operations++;
      
      try {
        const operationStart = Date.now();
        await operation(operations, changeSetId);
        const operationEnd = Date.now();
        
        successful++;
        
        // Progress reporting (throttled)
        if (operations % this.config.reportInterval === 0) {
          const elapsed = (Date.now() - globalStartTime.getTime()) / 1000;
          const threadOpsPerSec = successful / elapsed;
          console.log(`✅ Thread ${threadId}: [${operations}] ops | ${threadOpsPerSec.toFixed(1)} ops/sec | ${elapsed.toFixed(0)}s elapsed`);
        }
        
      } catch (error) {
        failed++;
        const errorMessage = error instanceof Error ? error.message : String(error);
        console.warn(`❌ Thread ${threadId}: Operation ${operations} failed: ${errorMessage}`);
        
        // Don't let errors stop the thread - continue with next operation
      }

      // Small delay to prevent overwhelming the API
      await sleep(10);
    }

    const threadEndTime = Date.now();
    const durationMs = threadEndTime - threadStartTime;

    const threadMetric: SoakThreadMetrics = {
      threadId,
      changeSetId,
      operations,
      successful,
      failed,
      durationMs,
    };

    console.log(`🏁 Thread ${threadId}: Completed ${operations} operations (${successful} successful, ${failed} failed) in ${(durationMs/1000).toFixed(1)}s`);
    
    return threadMetric;
  }

  /**
   * Report final test results (matching Python script style)
   */
  private reportFinalResults(metrics: SoakMetrics): void {
    console.log('\n============================================================');
    console.log('📊 FINAL RESULTS');
    console.log('============================================================');
    console.log(`Total Threads: ${metrics.threadMetrics.length}`);
    console.log(`Operations Completed: ${metrics.successfulOperations}/${metrics.totalOperations}`);
    console.log(`Failed Operations: ${metrics.failedOperations}`);
    console.log(`Success Rate: ${((metrics.successfulOperations / metrics.totalOperations) * 100).toFixed(1)}%`);
    console.log('');
    console.log('⏱️  TIMING BREAKDOWN:');
    console.log(`Total Test Runtime: ${(metrics.totalDurationMs / 1000).toFixed(1)}s`);
    console.log(`Operations per Second (aggregate): ${metrics.operationsPerSecond.toFixed(2)}`);
    console.log('');

    // Per-thread breakdown
    console.log('🧵 PER-THREAD BREAKDOWN:');
    metrics.threadMetrics.forEach(tm => {
      const threadOpsPerSec = tm.durationMs > 0 ? (tm.successful / tm.durationMs) * 1000 : 0;
      console.log(`  Thread ${tm.threadId}: ${tm.operations} ops (${tm.successful}✅/${tm.failed}❌) | ${threadOpsPerSec.toFixed(2)} ops/sec | ${(tm.durationMs/1000).toFixed(1)}s | CS: ${tm.changeSetId}`);
    });
    
    if (metrics.successfulOperations === metrics.totalOperations) {
      console.log(`\n🎉 SUCCESS: All ${metrics.totalOperations} operations completed successfully!`);
    } else {
      console.log(`\n⚠️  PARTIAL SUCCESS: ${metrics.successfulOperations} of ${metrics.totalOperations} operations completed`);
    }

    if (metrics.errors.length > 0) {
      console.log('\n❌ ERRORS ENCOUNTERED:');
      metrics.errors.slice(0, 5).forEach((error, index) => {
        console.log(`${index + 1}. Thread ${error.thread}, Op ${error.operation}: ${error.error}`);
      });
      if (metrics.errors.length > 5) {
        console.log(`... and ${metrics.errors.length - 5} more errors`);
      }
    }
  }
}

/**
 * Simple semaphore for controlling parallelism
 */
class Semaphore {
  private permits: number;
  private waiting: Array<() => void> = [];

  constructor(permits: number) {
    this.permits = permits;
  }

  async acquire(): Promise<() => void> {
    return new Promise((resolve) => {
      if (this.permits > 0) {
        this.permits--;
        resolve(() => this.release());
      } else {
        this.waiting.push(() => {
          this.permits--;
          resolve(() => this.release());
        });
      }
    });
  }

  private release(): void {
    this.permits++;
    if (this.waiting.length > 0) {
      const next = this.waiting.shift()!;
      next();
    }
  }
}

/**
 * Utility function to generate a unique test name with timestamp
 */
export function generateSoakTestName(prefix = 'soak-test'): string {
  const timestamp = new Date().toISOString().replace(/[^0-9]/g, '').slice(0, 14);
  const random = Math.random().toString(36).substring(2, 8);
  return `${prefix}-${timestamp}-${random}`;
}

/**
 * Utility function to sleep for a specified number of milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Parse command line arguments for soak test configuration
 */
export function parseSoakArgs(): SoakTestConfig & { testName?: string } {
  const args = Deno.args;
  const config: SoakTestConfig & { testName?: string } = {
    iterations: 100, // default iterations
    duration: 0, // unlimited by default
    parallelThreads: 1,
    cleanup: true,
    reportInterval: 25,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    const nextArg = args[i + 1];

    switch (arg) {
      case '--iterations':
      case '-i':
        if (nextArg && !isNaN(parseInt(nextArg))) {
          config.iterations = parseInt(nextArg);
          i++; // skip next arg
        }
        break;
      case '--duration':
      case '-d':
        if (nextArg) {
          const durationStr = nextArg;
          let duration = 0;
          
          // Parse duration with units: 5m, 30s, 300000ms, etc.
          if (durationStr.endsWith('m')) {
            duration = parseInt(durationStr.slice(0, -1)) * 60 * 1000;
          } else if (durationStr.endsWith('s')) {
            duration = parseInt(durationStr.slice(0, -1)) * 1000;
          } else if (durationStr.endsWith('ms')) {
            duration = parseInt(durationStr.slice(0, -2));
          } else {
            duration = parseInt(durationStr) * 1000; // assume seconds
          }
          
          config.duration = duration;
          i++; // skip next arg
        }
        break;
      case '--threads':
      case '-t':
        if (nextArg && !isNaN(parseInt(nextArg))) {
          config.parallelThreads = parseInt(nextArg);
          i++; // skip next arg
        }
        break;
      case '--test':
        if (nextArg) {
          config.testName = nextArg;
          i++; // skip next arg
        }
        break;
      case '--report-interval':
        if (nextArg && !isNaN(parseInt(nextArg))) {
          config.reportInterval = parseInt(nextArg);
          i++; // skip next arg
        }
        break;
      case '--no-cleanup':
        config.cleanup = false;
        break;
    }
  }

  return config;
}

/**
 * Print help for soak test command line arguments
 */
export function printSoakHelp(): void {
  console.log('Soak Test Arguments:');
  console.log('  --iterations, -i <num>    Maximum iterations per thread (default: 100)');
  console.log('  --duration, -d <time>     Maximum duration (5m, 30s, 300000ms) (default: unlimited)');
  console.log('  --threads, -t <num>       Number of parallel threads (default: 1)');
  console.log('  --test <name>             Specific test to run');
  console.log('  --report-interval <num>   Progress report interval (default: 25)');
  console.log('  --no-cleanup              Skip cleanup of resources');
  console.log('');
  console.log('Examples:');
  console.log('  deno task soak -- --duration 5m --threads 10');
  console.log('  deno task soak:1000 -- --iterations 500 --duration 10m --threads 5');
}