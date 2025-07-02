# Test file - should be skipped when using --skip-tests
import os

def test_with_secrets():
    # These are test secrets
    AWS_KEY = "AKIAIOSFODNN7TESTKEY"
    GITHUB_TOKEN = "ghp_test1234567890abcdefghijklmnopqrstuv"
    
    # Test API key
    api_key = "AIzaSyTestKeyForTestingPurposesOnly123"
    
    assert True