// Utility functions - no secrets here
function formatDate(date) {
  return date.toISOString().split('T')[0];
}

function calculateHash(input) {
  // This is just a hash, not a secret
  return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
}

const constants = {
  MAX_RETRIES: 3,
  TIMEOUT_MS: 5000,
  API_VERSION: "v2",
  // This looks like a key but it's just a UUID
  REQUEST_ID: "550e8400-e29b-41d4-a716-446655440000"
};

module.exports = { formatDate, calculateHash, constants };