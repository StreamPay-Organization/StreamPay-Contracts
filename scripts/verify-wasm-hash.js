#!/usr/bin/env node

const fs = require('fs');
const crypto = require('crypto');
const path = require('path');
const { execSync } = require('child_process');

// Configuration
const CONFIG = {
  contractName: 'stream_pay',
  wasmDir: './target/wasm32-unknown-unknown/release',
  expectedHashFile: './.deployed-hash',
  network: process.argv[2] || 'testnet'
};

// Colors for output
const colors = {
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  reset: '\x1b[0m'
};

function logInfo(msg) { console.log(`${colors.blue}ℹ${colors.reset} ${msg}`); }
function logSuccess(msg) { console.log(`${colors.green}✅${colors.reset} ${msg}`); }
function logWarning(msg) { console.log(`${colors.yellow}⚠️${colors.reset} ${msg}`); }
function logError(msg) { console.log(`${colors.red}❌${colors.reset} ${msg}`); }

function getWasmPath() {
  return path.join(CONFIG.wasmDir, `${CONFIG.contractName}.wasm`);
}

function calculateHash(filePath) {
  const fileBuffer = fs.readFileSync(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  return hashSum.digest('hex');
}

function getExpectedHash() {
  // Check environment variable
  if (process.env.EXPECTED_WASM_HASH) {
    return process.env.EXPECTED_WASM_HASH;
  }

  // Check network-specific hash file
  const networkHashFile = `${CONFIG.expectedHashFile}.${CONFIG.network}`;
  if (fs.existsSync(networkHashFile)) {
    return fs.readFileSync(networkHashFile, 'utf8').trim();
  }

  // Check default hash file
  if (fs.existsSync(CONFIG.expectedHashFile)) {
    return fs.readFileSync(CONFIG.expectedHashFile, 'utf8').trim();
  }

  // Check Cargo.toml for hash
  if (fs.existsSync('Cargo.toml')) {
    const cargoContent = fs.readFileSync('Cargo.toml', 'utf8');
    const match = cargoContent.match(/wasm-hash\s*=\s*"([^"]*)"/);
    if (match) {
      return match[1];
    }
  }

  return null;
}

function saveHash(hash) {
  const hashFile = `${CONFIG.expectedHashFile}.${CONFIG.network}`;
  fs.writeFileSync(hashFile, hash + '\n');
  logSuccess(`Hash saved to ${hashFile}`);
}

function buildWasm() {
  logInfo('Building WASM file...');
  try {
    execSync('make build', { stdio: 'inherit' });
    logSuccess('WASM built successfully');
  } catch (error) {
    logError('Failed to build WASM');
    process.exit(1);
  }
}

function verify() {
  console.log('');
  console.log('========================================');
  console.log('  WASM Hash Verification');
  console.log(`  Network: ${CONFIG.network}`);
  console.log(`  Contract: ${CONFIG.contractName}`);
  console.log('========================================');
  console.log('');

  const wasmPath = getWasmPath();

  // Check if WASM file exists
  if (!fs.existsSync(wasmPath)) {
    logError(`WASM file not found: ${wasmPath}`);
    logInfo('Run the script with --build to build the WASM file');
    process.exit(1);
  }

  // Get expected hash
  const expectedHash = getExpectedHash();
  if (!expectedHash) {
    logError('No expected hash found. Set EXPECTED_WASM_HASH or provide hash file.');
    process.exit(1);
  }

  // Calculate actual hash
  const actualHash = calculateHash(wasmPath);

  console.log(`📄 WASM File: ${wasmPath}`);
  console.log(`📊 Expected Hash: ${expectedHash}`);
  console.log(`📊 Actual Hash:   ${actualHash}`);
  console.log('');

  // Compare hashes
  if (actualHash === expectedHash) {
    logSuccess('Verification PASSED! Hashes match.');
    console.log('');
    console.log('The deployed contract matches the expected version.');
    process.exit(0);
  } else {
    logError('Verification FAILED! Hashes do not match.');
    console.log('');
    console.log('The deployed contract does NOT match the expected version.');
    console.log('This may indicate:');
    console.log('  - The contract was updated without updating the expected hash');
    console.log('  - A different version was deployed');
    console.log('  - The contract was tampered with');
    console.log('');
    console.log(`To update the expected hash:`);
    console.log(`  node scripts/verify-wasm-hash.js ${CONFIG.network} --save`);
    process.exit(1);
  }
}

// Parse arguments
const args = process.argv.slice(2);
let build = false;
let save = false;

args.forEach(arg => {
  if (arg === '--build') build = true;
  if (arg === '--save') save = true;
  if (arg === '--help') {
    console.log(`
Usage: node scripts/verify-wasm-hash.js [network] [options]

Options:
  network      Network to verify (mainnet|testnet|devnet) - default: testnet
  --build      Build the WASM file before verification
  --save       Save the computed hash as the expected hash
  --help       Show this help message

Examples:
  node scripts/verify-wasm-hash.js mainnet
  node scripts/verify-wasm-hash.js testnet --build
  node scripts/verify-wasm-hash.js devnet --save
    `);
    process.exit(0);
  }
});

// Update network from args
const networkArg = args.find(a => ['mainnet', 'testnet', 'devnet'].includes(a));
if (networkArg) {
  CONFIG.network = networkArg;
}

// Build if requested
if (build) {
  buildWasm();
}

// Save hash if requested
if (save) {
  const wasmPath = getWasmPath();
  if (fs.existsSync(wasmPath)) {
    const hash = calculateHash(wasmPath);
    saveHash(hash);
    console.log(`Hash: ${hash}`);
  } else {
    logError('Cannot save hash: WASM file not found');
    logInfo('Build the WASM file first: node scripts/verify-wasm-hash.js --build');
    process.exit(1);
  }
}

// Run verification
verify();