# Node.js and CLI Patterns Reference

## Philosophy: CLI-First, Node.js for Complex Logic

**Emphasize modern tooling:**
- Use CLI tools liberally (gh, aws, npm, git, jq, etc.)
- Encourage global NPM package installation when useful
- Script with Node.js v24+ (not Python)
- Provide complete, runnable commands
- Show command chaining patterns

## CLI Tools to Leverage

### GitHub CLI (`gh`)

```bash
# View repository information
gh repo view --json name,description,url

# List and view pull requests
gh pr list --state open --limit 10
gh pr view 123 --json title,body,commits
gh pr diff 123

# Create pull requests
gh pr create --title "Feature: Add auth" --body "Implementation details..."

# Work with issues
gh issue list --label bug --limit 20
gh issue view 456
gh issue create --title "Bug: Login fails" --body "Description..."

# GitHub Actions workflows
gh run list --limit 10
gh run view 789
gh workflow run deploy.yml
```

### AWS CLI (`aws`)

```bash
# S3 operations
aws s3 ls s3://bucket-name/
aws s3 cp local-file.txt s3://bucket-name/
aws s3 sync ./dist/ s3://bucket-name/

# Lambda operations
aws lambda list-functions
aws lambda invoke --function-name my-function output.json
aws lambda get-function --function-name my-function

# DynamoDB operations
aws dynamodb scan --table-name MyTable
aws dynamodb get-item --table-name MyTable --key '{"id": {"S": "123"}}'

# CloudWatch Logs
aws logs tail /aws/lambda/my-function --follow
aws logs get-log-events --log-group-name /aws/lambda/my-function --log-stream-name 2024/01/01/latest
```

### NPM CLI

```bash
# Global package installation
npm install -g csv-parse
npm install -g aws-sdk
npm install -g lodash

# View package info
npm view package-name version
npm view package-name dependencies

# Local package management
npm install package-name
npm install --save-dev package-name
npm update

# Script running
npm run build
npm run test
npm run deploy
```

### Git CLI

```bash
# Repository inspection
git status
git log --oneline -10
git log --author="name" --since="2024-01-01"
git diff main...feature-branch
git blame filename.js

# Branch operations
git branch --list
git branch -d old-branch
git checkout -b new-feature

# Commit history
git show HEAD
git show abc123
git log --graph --all --oneline
```

### jq (JSON processor)

```bash
# Extract fields
echo '{"name": "John", "age": 30}' | jq '.name'

# Filter arrays
cat data.json | jq '.[] | select(.active == true)'

# Transform structure
cat input.json | jq '{id: .id, name: .user.name}'

# Combine with other CLI tools
gh pr view 123 --json title,author | jq '.author.login'
aws lambda get-function --function-name my-fn | jq '.Configuration.Environment'
```

### Command Chaining

```bash
# Sequential with &&
gh pr view 123 --json commits && git diff main...pr-123 && npm test

# Piping output
aws lambda list-functions | jq '.Functions[].FunctionName' | xargs -I {} echo "Function: {}"

# With error handling
gh pr diff 123 > changes.diff && cat changes.diff | wc -l && echo "Lines changed"

# Background processes
npm run dev &
aws logs tail /aws/lambda/function --follow &
```

## Node.js v24+ Patterns

### File Operations with ESM

```javascript
#!/usr/bin/env node
import { readFile, writeFile, readdir, stat } from 'fs/promises';
import { join } from 'path';

// Read file
const content = await readFile('input.txt', 'utf-8');
console.log(content);

// Write file
await writeFile('output.txt', content.toUpperCase());

// Read directory
const files = await readdir('./data');
for (const file of files) {
  const filePath = join('./data', file);
  const stats = await stat(filePath);
  console.log(`${file}: ${stats.size} bytes`);
}
```

### Running CLI Commands from Node.js

```javascript
#!/usr/bin/env node
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Simple command
const { stdout, stderr } = await execAsync('gh repo view --json name');
const repo = JSON.parse(stdout);
console.log(repo.name);

// With error handling
try {
  const { stdout } = await execAsync('aws lambda list-functions');
  const functions = JSON.parse(stdout);
  console.log(`Found ${functions.Functions.length} functions`);
} catch (error) {
  console.error(`Command failed: ${error.message}`);
}

// Multiple commands
const commands = [
  'git status',
  'npm test',
  'gh pr list'
];

for (const cmd of commands) {
  try {
    const { stdout } = await execAsync(cmd);
    console.log(`${cmd}:\n${stdout}`);
  } catch (error) {
    console.error(`${cmd} failed: ${error.message}`);
  }
}
```

### Processing JSON Data

```javascript
#!/usr/bin/env node
import { readFile, writeFile } from 'fs/promises';

// Parse JSON
const data = JSON.parse(await readFile('data.json', 'utf-8'));

// Filter
const active = data.filter(item => item.active);

// Map/transform
const names = data.map(item => ({
  id: item.id,
  fullName: `${item.firstName} ${item.lastName}`
}));

// Reduce/aggregate
const total = data.reduce((sum, item) => sum + item.value, 0);

// Write result
await writeFile('output.json', JSON.stringify(active, null, 2));
```

### Working with CSV (using csv-parse)

```javascript
#!/usr/bin/env node
import { readFile, writeFile } from 'fs/promises';
import { parse } from 'csv-parse/sync';
import { stringify } from 'csv-stringify/sync';

// Read and parse CSV
const content = await readFile('data.csv', 'utf-8');
const records = parse(content, {
  columns: true,
  skip_empty_lines: true
});

// Process records
const filtered = records.filter(row => row.status === 'active');

// Transform
const transformed = filtered.map(row => ({
  ...row,
  processedDate: new Date().toISOString()
}));

// Write CSV
const output = stringify(transformed, { header: true });
await writeFile('output.csv', output);
```

### Async/Await Patterns

```javascript
#!/usr/bin/env node
import { readFile } from 'fs/promises';

// Sequential processing
async function processFiles(filePaths) {
  const results = [];
  for (const path of filePaths) {
    const content = await readFile(path, 'utf-8');
    results.push(content.length);
  }
  return results;
}

// Parallel processing
async function processFilesParallel(filePaths) {
  const promises = filePaths.map(path => readFile(path, 'utf-8'));
  const contents = await Promise.all(promises);
  return contents.map(c => c.length);
}

// Usage
const files = ['file1.txt', 'file2.txt', 'file3.txt'];
const sizes = await processFilesParallel(files);
console.log(sizes);
```

### Error Handling

```javascript
#!/usr/bin/env node
import { readFile } from 'fs/promises';

// Try-catch for individual operations
async function safeReadFile(path) {
  try {
    return await readFile(path, 'utf-8');
  } catch (error) {
    console.error(`Failed to read ${path}: ${error.message}`);
    return null;
  }
}

// Promise.allSettled for multiple operations
async function readMultipleFiles(paths) {
  const results = await Promise.allSettled(
    paths.map(path => readFile(path, 'utf-8'))
  );

  for (const [index, result] of results.entries()) {
    if (result.status === 'fulfilled') {
      console.log(`${paths[index]}: ${result.value.length} chars`);
    } else {
      console.error(`${paths[index]}: ${result.reason.message}`);
    }
  }
}
```

### Working with APIs

```javascript
#!/usr/bin/env node

// Fetch API (built-in Node.js 24+)
const response = await fetch('https://api.github.com/repos/owner/repo');
const data = await response.json();
console.log(data.name);

// With error handling
async function fetchRepo(owner, repo) {
  try {
    const url = `https://api.github.com/repos/${owner}/${repo}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error(`Failed to fetch repo: ${error.message}`);
    return null;
  }
}

// POST request
async function createIssue(owner, repo, token, title, body) {
  const url = `https://api.github.com/repos/${owner}/${repo}/issues`;
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Authorization': `token ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ title, body })
  });

  return await response.json();
}
```

### Script Template

```javascript
#!/usr/bin/env node
import { readFile, writeFile } from 'fs/promises';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Configuration
const CONFIG = {
  inputFile: 'input.json',
  outputFile: 'output.json',
  maxRetries: 3
};

// Helper functions
async function loadData(filePath) {
  const content = await readFile(filePath, 'utf-8');
  return JSON.parse(content);
}

async function saveData(filePath, data) {
  await writeFile(filePath, JSON.stringify(data, null, 2));
}

async function runCommand(cmd) {
  const { stdout, stderr } = await execAsync(cmd);
  return stdout.trim();
}

// Main logic
async function main() {
  try {
    console.log('Loading data...');
    const data = await loadData(CONFIG.inputFile);

    console.log('Processing...');
    const processed = data.map(item => ({
      ...item,
      processed: true,
      timestamp: new Date().toISOString()
    }));

    console.log('Saving results...');
    await saveData(CONFIG.outputFile, processed);

    console.log('Done!');
  } catch (error) {
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
}

// Run
main();
```

## Why Node.js Over Python

**Benefits of Node.js for Claude Code skills:**

1. **Consistency**: Same language as NPM ecosystem
2. **Modern async/await**: Clean, readable async code
3. **Built-in JSON**: Native JSON parsing and stringification
4. **Active LTS**: Node.js v24+ has excellent support
5. **ESM imports**: Modern module system
6. **CLI integration**: Easy to call CLI tools with exec
7. **No virtual env**: No dependency environment management needed
8. **Fast startup**: Quick execution for scripts

**Avoid Python:**
- Additional dependency (Python installation)
- Virtual environment complexity
- Type system differences
- Less consistent with NPM/JS ecosystem

## Global NPM Packages to Consider

**Data processing:**
```bash
npm install -g csv-parse csv-stringify
npm install -g lodash
npm install -g ramda
```

**AWS tools:**
```bash
npm install -g aws-sdk
npm install -g serverless
```

**Testing:**
```bash
npm install -g jest
npm install -g mocha
```

**Build tools:**
```bash
npm install -g esbuild
npm install -g webpack-cli
```

**Utilities:**
```bash
npm install -g prettier
npm install -g eslint
npm install -g nodemon
```

## Complete Examples

### Example: GitHub PR Analyzer

```javascript
#!/usr/bin/env node
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function analyzePR(prNumber) {
  // Get PR details
  const { stdout: prData } = await execAsync(
    `gh pr view ${prNumber} --json title,author,commits,additions,deletions`
  );
  const pr = JSON.parse(prData);

  // Get diff
  const { stdout: diff } = await execAsync(`gh pr diff ${prNumber}`);

  // Analyze
  const analysis = {
    title: pr.title,
    author: pr.author.login,
    commits: pr.commits.length,
    linesAdded: pr.additions,
    linesDeleted: pr.deletions,
    totalChanges: pr.additions + pr.deletions,
    diffLines: diff.split('\n').length
  };

  console.log('PR Analysis:');
  console.log(JSON.stringify(analysis, null, 2));
}

const prNumber = process.argv[2];
if (!prNumber) {
  console.error('Usage: analyze-pr.js <pr-number>');
  process.exit(1);
}

analyzePR(prNumber);
```

### Example: AWS Lambda Log Analyzer

```javascript
#!/usr/bin/env node
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function analyzeLogs(functionName, minutes = 60) {
  const { stdout } = await execAsync(
    `aws logs tail /aws/lambda/${functionName} --since ${minutes}m`
  );

  const lines = stdout.split('\n').filter(Boolean);

  const analysis = {
    total: lines.length,
    errors: lines.filter(l => l.includes('ERROR')).length,
    warnings: lines.filter(l => l.includes('WARN')).length,
    info: lines.filter(l => l.includes('INFO')).length
  };

  console.log(`Analysis for ${functionName}:`);
  console.log(`Total logs: ${analysis.total}`);
  console.log(`Errors: ${analysis.errors}`);
  console.log(`Warnings: ${analysis.warnings}`);
  console.log(`Info: ${analysis.info}`);

  if (analysis.errors > 0) {
    console.log('\nRecent errors:');
    lines
      .filter(l => l.includes('ERROR'))
      .slice(-5)
      .forEach(l => console.log(l));
  }
}

const [functionName, minutes] = process.argv.slice(2);
if (!functionName) {
  console.error('Usage: analyze-logs.js <function-name> [minutes]');
  process.exit(1);
}

analyzeLogs(functionName, minutes ? parseInt(minutes) : 60);
```

## Best Practices

1. **Use ESM imports** - `import` not `require`
2. **Use async/await** - Cleaner than callbacks or raw promises
3. **Make scripts executable** - Add `#!/usr/bin/env node` shebang
4. **Handle errors gracefully** - Try-catch with meaningful messages
5. **Accept CLI arguments** - Use `process.argv` for flexibility
6. **Provide usage info** - Show help when arguments are missing
7. **Use promisify for exec** - Makes CLI command execution clean
8. **Log progress** - Console.log intermediate steps
9. **Exit with codes** - `process.exit(1)` on errors
10. **Keep scripts focused** - One clear purpose per script
