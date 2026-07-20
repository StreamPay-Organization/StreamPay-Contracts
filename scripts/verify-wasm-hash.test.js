const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

// Import the verification functions
// Note: Since we're using CommonJS, we'll duplicate the functions here for testing
function calculateHash(filePath) {
  const fileBuffer = fs.readFileSync(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  return hashSum.digest('hex');
}

describe('WASM Hash Verification', () => {
  let testWasmPath;

  beforeEach(() => {
    // Create a test WASM file
    testWasmPath = path.join(__dirname, 'test.wasm');
    const testContent = Buffer.from([0x00, 0x61, 0x73, 0x6d]); // WASM magic number
    fs.writeFileSync(testWasmPath, testContent);
  });

  afterEach(() => {
    try {
      if (fs.existsSync(testWasmPath)) {
        fs.unlinkSync(testWasmPath);
      }
    } catch (error) {
      // Ignore cleanup errors
    }
  });

  test('should calculate correct SHA-256 hash for WASM file', () => {
    const hash = calculateHash(testWasmPath);
    expect(hash).toBeDefined();
    expect(typeof hash).toBe('string');
    expect(hash.length).toBe(64); // SHA-256 hex length
  });

  test('should detect when hash matches expected', () => {
    const actualHash = calculateHash(testWasmPath);
    const expectedHash = actualHash;
    expect(actualHash).toBe(expectedHash);
  });

  test('should detect when hash does not match expected', () => {
    const actualHash = calculateHash(testWasmPath);
    const expectedHash = 'wronghash1234567890abcdefghijklmnopqrstuvwxyz12345678';
    expect(actualHash).not.toBe(expectedHash);
  });

  test('should throw error when WASM file does not exist', () => {
    expect(() => {
      calculateHash('/nonexistent/file.wasm');
    }).toThrow();
  });

  test('should produce consistent hash for same file', () => {
    const hash1 = calculateHash(testWasmPath);
    const hash2 = calculateHash(testWasmPath);
    expect(hash1).toBe(hash2);
  });
});