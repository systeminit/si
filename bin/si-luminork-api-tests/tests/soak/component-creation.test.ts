/**
 * Component Creation Soak Test
 *
 * Parameterized soak test for creating components where iteration number = component number.
 * Each thread creates components numbered sequentially within its own changeset.
 *
 * Usage examples:
 *   deno task soak:components -- --iterations 1000 --threads 10 --duration 5m
 *   deno task soak:components -- --iterations 500 --threads 5
 *   deno task soak:components -- --duration 2m --threads 3
 *
 * Each thread will create components like:
 *   Thread 1: ec2-001, ec2-002, ec2-003, ... up to iteration limit or duration
 *   Thread 2: ec2-001, ec2-002, ec2-003, ... (in its own changeset)
 */

import { assertEquals } from 'https://deno.land/std@0.220.1/assert/mod.ts';
import {
  cleanupTestResources,
  ConfigError,
  createTestClient,
} from '../../src/test-utils.ts';
import { 
  SoakTestRunner, 
  generateSoakTestName, 
  parseSoakArgs,
  printSoakHelp 
} from '../../src/soak-utils.ts';

Deno.test('Soak Test - Component Creation', async () => {
  // Check for help flag
  if (Deno.args.includes('--help') || Deno.args.includes('-h')) {
    printSoakHelp();
    console.log('\nComponent Creation Test Specific Info:');
    console.log('  - Iteration number = component number to create');
    console.log('  - Each thread creates components in its own changeset');
    console.log('  - Components named: ec2-001, ec2-002, ec2-003, etc.');
    console.log('  - Use --iterations to set max components per thread');
    console.log('  - Use --duration to set max time per thread');
    return;
  }

  try {
    const { api, config } = await createTestClient();

    // Parse command line arguments
    const soakConfig = parseSoakArgs();

    console.log('🚀 Starting Component Creation Soak Test');
    console.log('========================================');
    console.log(`Configuration:`);
    console.log(`  Max Components per Thread: ${soakConfig.iterations || 'unlimited'}`);
    console.log(`  Max Duration: ${soakConfig.duration ? (soakConfig.duration / 1000) + 's' : 'unlimited'}`);
    console.log(`  Parallel Threads: ${soakConfig.parallelThreads}`);
    console.log(`  Report Interval: ${soakConfig.reportInterval}`);
    console.log(`  Cleanup: ${soakConfig.cleanup ? 'Yes' : 'No'}`);

    const allChangeSetIds: string[] = [];

    try {
      const runner = new SoakTestRunner(soakConfig);

      // Changeset factory - creates one changeset per thread
      const changeSetFactory = async (threadId: number): Promise<string> => {
        const changeSetName = generateSoakTestName(`components-t${threadId}`);
        console.log(`📝 Thread ${threadId}: Creating changeset ${changeSetName}`);
        
        const response = await api.changeSets.createChangeSet(config.workspaceId, {
          changeSetName,
        });

        assertEquals(response.status, 200);
        const changeSetId = response.data.changeSet.id;
        allChangeSetIds.push(changeSetId);
        
        return changeSetId;
      };

      // Component creation operation with immediate verification
      const componentCreationOp = async (componentNumber: number, changeSetId: string) => {
        const componentName = `ec2-${componentNumber.toString().padStart(3, '0')}`;
        
        // Step 1: Create the component
        const createResponse = await api.components.createComponent(
          config.workspaceId,
          changeSetId,
          {
            name: componentName,
            schemaName: 'AWS::EC2::Instance',
          }
        );

        assertEquals(createResponse.status, 200);
        const componentId = createResponse.data.component.id;

        // Step 2: Immediately list components to verify 200 status
        const listResponse = await api.components.listComponents(config.workspaceId, changeSetId);
        assertEquals(listResponse.status, 200);

        // Step 3: Immediately GET the component by ID to verify 200 status
        const getResponse = await api.components.getComponent(
          config.workspaceId,
          changeSetId,
          componentId
        );
        
        assertEquals(getResponse.status, 200);
        
        return {
          created: createResponse.data,
          listVerified: true,
          getVerified: true,
        };
      };

      // Run the soak test (each operation includes immediate verification)
      const metrics = await runner.run(changeSetFactory, componentCreationOp);

      // Results summary
      console.log('\n📊 COMPONENT CREATION RESULTS:');
      console.log(`Total Components Created: ${metrics.successfulOperations}`);
      console.log(`Failed Component Creations: ${metrics.failedOperations}`);
      console.log(`Success Rate: ${((metrics.successfulOperations / metrics.totalOperations) * 100).toFixed(1)}%`);
      console.log(`Total Changesets Used: ${allChangeSetIds.length}`);
      console.log(`Average Components per Changeset: ${(metrics.successfulOperations / allChangeSetIds.length).toFixed(1)}`);

      // Performance analysis per thread
      if (metrics.threadMetrics.length > 1) {
        const threadPerformances = metrics.threadMetrics.map(tm => 
          tm.durationMs > 0 ? (tm.successful / tm.durationMs) * 1000 : 0
        );
        const avgThreadPerf = threadPerformances.reduce((sum, perf) => sum + perf, 0) / threadPerformances.length;
        const minThreadPerf = Math.min(...threadPerformances);
        const maxThreadPerf = Math.max(...threadPerformances);

        console.log(`\n📈 THREAD PERFORMANCE ANALYSIS:`);
        console.log(`  Average Thread Performance: ${avgThreadPerf.toFixed(2)} components/sec`);
        console.log(`  Best Thread Performance: ${maxThreadPerf.toFixed(2)} components/sec`);
        console.log(`  Worst Thread Performance: ${minThreadPerf.toFixed(2)} components/sec`);
        
        // Show per-thread component counts
        console.log(`\n🧵 COMPONENTS PER THREAD:`);
        metrics.threadMetrics.forEach(tm => {
          console.log(`  Thread ${tm.threadId}: ${tm.successful} components (ec2-001 to ec2-${tm.successful.toString().padStart(3, '0')})`);
        });
      }

      // Test assertions
      assertEquals(metrics.threadMetrics.length, soakConfig.parallelThreads, 'Should have metrics for all threads');
      
      // At least 95% success rate for creation
      const successRate = (metrics.successfulOperations / metrics.totalOperations) * 100;
      if (successRate < 95) {
        console.warn(`⚠️  Success rate ${successRate.toFixed(1)}% is below 95% threshold`);
      } else {
        console.log(`✅ Success rate ${successRate.toFixed(1)}% meets threshold`);
      }

      // Performance threshold - at least 0.5 components/sec aggregate
      if (metrics.operationsPerSecond < 0.5) {
        console.warn(`⚠️  Aggregate performance ${metrics.operationsPerSecond.toFixed(2)} components/sec may be low`);
      } else {
        console.log(`✅ Performance acceptable: ${metrics.operationsPerSecond.toFixed(2)} components/sec`);
      }

      // Report maximum components created in any single thread
      if (metrics.threadMetrics.length > 0) {
        const maxComponents = Math.max(...metrics.threadMetrics.map(tm => tm.successful));
        const minComponents = Math.min(...metrics.threadMetrics.map(tm => tm.successful));
        console.log(`\n📦 COMPONENT RANGE:`);
        console.log(`  Maximum components in a thread: ${maxComponents} (ec2-001 to ec2-${maxComponents.toString().padStart(3, '0')})`);
        console.log(`  Minimum components in a thread: ${minComponents} (ec2-001 to ec2-${minComponents.toString().padStart(3, '0')})`);
      }

    } finally {
      // Clean up resources if requested
      if (soakConfig.cleanup && allChangeSetIds.length > 0) {
        console.log(`\n🧹 Cleaning up ${allChangeSetIds.length} changesets...`);
        await cleanupTestResources(api, config.workspaceId, allChangeSetIds);
        console.log('✅ Cleanup completed');
      } else if (!soakConfig.cleanup) {
        console.log(`\n📝 Skipping cleanup as requested. Changesets created: ${allChangeSetIds.length}`);
        allChangeSetIds.forEach((id, index) => {
          console.log(`  ${index + 1}. ${id}`);
        });
      }
    }

  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping component creation soak test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});

Deno.test('Soak Test - Quick Component Creation (Fixed)', async () => {
  try {
    const { api, config } = await createTestClient();

    console.log('🚀 Quick Component Creation Test');
    console.log('================================');

    const allChangeSetIds: string[] = [];

    try {
      // Fixed configuration for quick test - create 25 components per thread
      const runner = new SoakTestRunner({
        iterations: 25,
        duration: 2 * 60 * 1000, // 2 minutes max
        parallelThreads: 2,
        cleanup: true,
        reportInterval: 5,
      });

      // Changeset factory
      const changeSetFactory = async (threadId: number): Promise<string> => {
        const changeSetName = generateSoakTestName(`quick-components-t${threadId}`);
        const response = await api.changeSets.createChangeSet(config.workspaceId, {
          changeSetName,
        });
        assertEquals(response.status, 200);
        const changeSetId = response.data.changeSet.id;
        allChangeSetIds.push(changeSetId);
        return changeSetId;
      };

      // Component creation operation
      const componentCreationOp = async (componentNumber: number, changeSetId: string) => {
        const componentName = `quick-ec2-${componentNumber.toString().padStart(2, '0')}`;
        const response = await api.components.createComponent(
          config.workspaceId,
          changeSetId,
          {
            name: componentName,
            schemaName: 'AWS::EC2::Instance',
          }
        );
        assertEquals(response.status, 200);
        return response.data;
      };

      // Run the test
      const metrics = await runner.run(changeSetFactory, componentCreationOp);

      console.log(`✅ Created ${metrics.successfulOperations} components total`);
      console.log(`📊 Performance: ${metrics.operationsPerSecond.toFixed(2)} components/sec`);
      
      // Show what was created per thread
      metrics.threadMetrics.forEach(tm => {
        console.log(`  Thread ${tm.threadId}: ${tm.successful} components (quick-ec2-01 to quick-ec2-${tm.successful.toString().padStart(2, '0')})`);
      });

      // Quick test assertions
      assertEquals(metrics.threadMetrics.length, 2, 'Should have 2 threads');
      
      const successRate = (metrics.successfulOperations / metrics.totalOperations) * 100;
      if (successRate >= 95) {
        console.log(`✅ Quick test passed: ${successRate.toFixed(1)}% success rate`);
      } else {
        console.warn(`⚠️  Quick test warning: ${successRate.toFixed(1)}% success rate below 95%`);
      }

    } finally {
      await cleanupTestResources(api, config.workspaceId, allChangeSetIds);
    }

  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping quick component creation test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});