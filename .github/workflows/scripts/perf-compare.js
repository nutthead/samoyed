#!/usr/bin/env node

/**
 * Performance Comparison Script for Samoid
 * 
 * This script handles:
 * - Storing benchmark results with metadata
 * - Comparing current results against baseline/historical data
 * - Generating performance comparison reports
 * - Detecting performance regressions and improvements
 */

const fs = require('fs');
const path = require('path');

// Performance thresholds for regression detection
const THRESHOLDS = {
  CRITICAL_REGRESSION: 0.20,  // 20% performance degradation
  WARNING_REGRESSION: 0.10,   // 10% performance degradation  
  IMPROVEMENT: 0.10,          // 10% performance improvement
  ACCEPTABLE_VARIANCE: 0.10   // ±10% considered normal fluctuation
};

// Metrics to track
const METRICS = {
  HOOK_OVERHEAD_MS: 'hook_execution_overhead_ms',
  STARTUP_TIME_MS: 'startup_time_ms',
  BINARY_SIZE_BYTES: 'binary_size_bytes',
  MEMORY_USAGE_KB: 'memory_usage_kb',
  FILESYSTEM_OPS_US: 'filesystem_operations_us'
};

class PerformanceTracker {
  constructor(dataDir = './.perf-data') {
    this.dataDir = dataDir;
    this.ensureDataDir();
  }

  ensureDataDir() {
    if (!fs.existsSync(this.dataDir)) {
      fs.mkdirSync(this.dataDir, { recursive: true });
    }
  }

  /**
   * Store performance results with metadata
   */
  storeResults(results) {
    const timestamp = new Date().toISOString();
    const commitSha = process.env.GITHUB_SHA || 'unknown';
    const branch = process.env.GITHUB_REF_NAME || 'unknown';
    const actor = process.env.GITHUB_ACTOR || 'unknown';
    const runId = process.env.GITHUB_RUN_ID || Date.now().toString();

    const record = {
      timestamp,
      commitSha,
      branch,
      actor,
      runId,
      environment: {
        os: process.env.RUNNER_OS || 'unknown',
        arch: process.env.RUNNER_ARCH || 'unknown',
        runner: 'github-actions'
      },
      metrics: results
    };

    // Store individual result
    const filename = `perf-${timestamp.replace(/[:.]/g, '-')}-${commitSha.slice(0, 8)}.json`;
    const filepath = path.join(this.dataDir, filename);
    fs.writeFileSync(filepath, JSON.stringify(record, null, 2));

    // Update latest results
    const latestPath = path.join(this.dataDir, 'latest.json');
    fs.writeFileSync(latestPath, JSON.stringify(record, null, 2));

    console.log(`✅ Performance results stored: ${filename}`);
    return record;
  }

  /**
   * Get baseline performance data (from master branch or historical average)
   */
  getBaseline() {
    const baselinePath = path.join(this.dataDir, 'baseline.json');
    
    if (fs.existsSync(baselinePath)) {
      return JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
    }

    // If no explicit baseline, try to find master branch results
    const files = fs.readdirSync(this.dataDir)
      .filter(f => f.startsWith('perf-') && f.endsWith('.json'))
      .sort()
      .reverse(); // Most recent first

    for (const file of files.slice(0, 10)) { // Check last 10 results
      try {
        const data = JSON.parse(fs.readFileSync(path.join(this.dataDir, file), 'utf8'));
        if (data.branch === 'master' || data.branch === 'main') {
          return data;
        }
      } catch (e) {
        // Skip invalid files
      }
    }

    return null;
  }

  /**
   * Compare current results against baseline
   */
  compareResults(current, baseline) {
    if (!baseline) {
      return {
        status: 'no-baseline',
        message: 'No baseline data available for comparison',
        comparisons: []
      };
    }

    const comparisons = [];
    let hasRegression = false;
    let hasImprovement = false;

    for (const [metricKey, metricName] of Object.entries(METRICS)) {
      const currentValue = current.metrics[metricName];
      const baselineValue = baseline.metrics[metricName];

      if (currentValue !== undefined && baselineValue !== undefined) {
        const change = (currentValue - baselineValue) / baselineValue;
        const changePercent = change * 100;

        let status = 'stable';
        let severity = 'info';

        if (Math.abs(change) > THRESHOLDS.CRITICAL_REGRESSION) {
          status = change > 0 ? 'critical_regression' : 'critical_improvement';
          severity = change > 0 ? 'error' : 'success';
          if (change > 0) hasRegression = true;
          if (change < 0) hasImprovement = true;
        } else if (Math.abs(change) > THRESHOLDS.WARNING_REGRESSION) {
          status = change > 0 ? 'warning_regression' : 'improvement';
          severity = change > 0 ? 'warning' : 'success';
          if (change > 0) hasRegression = true;
          if (change < 0) hasImprovement = true;
        } else if (Math.abs(change) > THRESHOLDS.IMPROVEMENT) {
          status = change > 0 ? 'regression' : 'improvement';
          severity = change > 0 ? 'warning' : 'success';
          if (change < 0) hasImprovement = true;
        }

        comparisons.push({
          metric: metricName,
          current: currentValue,
          baseline: baselineValue,
          change,
          changePercent,
          status,
          severity
        });
      }
    }

    const overallStatus = hasRegression ? 'regression' : hasImprovement ? 'improvement' : 'stable';

    return {
      status: overallStatus,
      hasRegression,
      hasImprovement,
      comparisons
    };
  }

  /**
   * Generate performance comparison report
   */
  generateReport(comparison, current, baseline) {
    let report = '# 📊 Performance Comparison Report\n\n';

    // Overall status
    const statusEmoji = {
      'no-baseline': 'ℹ️',
      'stable': '✅',
      'improvement': '📈',
      'regression': '⚠️'
    }[comparison.status] || '❓';

    report += `**Overall Status:** ${statusEmoji} ${comparison.status.toUpperCase()}\n\n`;

    if (comparison.message) {
      report += `${comparison.message}\n\n`;
      return report;
    }

    // Comparison table
    report += '## 📈 Performance Metrics Comparison\n\n';
    report += '| Metric | Current | Baseline | Change | Status |\n';
    report += '|--------|---------|----------|--------|---------|\n';

    for (const comp of comparison.comparisons) {
      const statusEmoji = {
        'stable': '✅',
        'improvement': '📈',
        'critical_improvement': '🚀',
        'regression': '⚠️',
        'warning_regression': '⚠️',
        'critical_regression': '🚨'
      }[comp.status] || '❓';

      const changeStr = comp.changePercent >= 0 ? `+${comp.changePercent.toFixed(1)}%` : `${comp.changePercent.toFixed(1)}%`;
      
      report += `| ${comp.metric} | ${comp.current} | ${comp.baseline} | ${changeStr} | ${statusEmoji} ${comp.status} |\n`;
    }

    report += '\n';

    // Detailed analysis
    if (comparison.hasRegression) {
      report += '## ⚠️ Performance Regressions Detected\n\n';
      const regressions = comparison.comparisons.filter(c => c.status.includes('regression'));
      for (const reg of regressions) {
        const severity = reg.status.includes('critical') ? '🚨 **CRITICAL**' : '⚠️ **WARNING**';
        report += `${severity}: ${reg.metric} degraded by ${Math.abs(reg.changePercent).toFixed(1)}%\n`;
      }
      report += '\n';
    }

    if (comparison.hasImprovement) {
      report += '## 📈 Performance Improvements\n\n';
      const improvements = comparison.comparisons.filter(c => c.status.includes('improvement'));
      for (const imp of improvements) {
        const level = imp.status.includes('critical') ? '🚀 **SIGNIFICANT**' : '📈 **IMPROVEMENT**';
        report += `${level}: ${imp.metric} improved by ${Math.abs(imp.changePercent).toFixed(1)}%\n`;
      }
      report += '\n';
    }

    // Metadata
    report += '## ℹ️ Test Environment\n\n';
    report += `- **Current Commit:** \`${current.commitSha}\`\n`;
    report += `- **Baseline Commit:** \`${baseline.commitSha}\`\n`;
    report += `- **Branch:** ${current.branch}\n`;
    report += `- **Environment:** ${current.environment.os} ${current.environment.arch}\n`;
    report += `- **Timestamp:** ${current.timestamp}\n\n`;

    return report;
  }
}

// CLI Interface
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  const tracker = new PerformanceTracker();

  switch (command) {
    case 'store': {
      const resultsFile = args[1];
      if (!resultsFile || !fs.existsSync(resultsFile)) {
        console.error('❌ Results file not found:', resultsFile);
        process.exit(1);
      }

      const results = JSON.parse(fs.readFileSync(resultsFile, 'utf8'));
      tracker.storeResults(results);
      break;
    }

    case 'compare': {
      const currentFile = args[1];
      if (!currentFile || !fs.existsSync(currentFile)) {
        console.error('❌ Current results file not found:', currentFile);
        process.exit(1);
      }

      const current = JSON.parse(fs.readFileSync(currentFile, 'utf8'));
      const baseline = tracker.getBaseline();
      const comparison = tracker.compareResults(current, baseline);
      const report = tracker.generateReport(comparison, current, baseline);

      // Write report
      const reportFile = 'performance-comparison.md';
      fs.writeFileSync(reportFile, report);
      console.log(`📊 Performance comparison report generated: ${reportFile}`);

      // Exit with error code if there are critical regressions
      const criticalRegressions = comparison.comparisons?.filter(c => c.status === 'critical_regression') || [];
      if (criticalRegressions.length > 0) {
        console.error(`🚨 ${criticalRegressions.length} critical performance regression(s) detected!`);
        process.exit(1);
      }

      break;
    }

    case 'set-baseline': {
      const baselineFile = args[1];
      if (!baselineFile || !fs.existsSync(baselineFile)) {
        console.error('❌ Baseline file not found:', baselineFile);
        process.exit(1);
      }

      const baseline = JSON.parse(fs.readFileSync(baselineFile, 'utf8'));
      const baselinePath = path.join(tracker.dataDir, 'baseline.json');
      fs.writeFileSync(baselinePath, JSON.stringify(baseline, null, 2));
      console.log(`✅ Baseline set from: ${baselineFile}`);
      break;
    }

    default:
      console.log(`
Usage: node perf-compare.js <command> [args]

Commands:
  store <results.json>         Store performance results with metadata
  compare <current.json>       Compare current results against baseline
  set-baseline <baseline.json> Set baseline for future comparisons

Examples:
  node perf-compare.js store ./results.json
  node perf-compare.js compare ./current-results.json
  node perf-compare.js set-baseline ./master-results.json
`);
      process.exit(1);
  }
}

if (require.main === module) {
  main().catch(error => {
    console.error('❌ Error:', error.message);
    process.exit(1);
  });
}

module.exports = { PerformanceTracker, THRESHOLDS, METRICS };