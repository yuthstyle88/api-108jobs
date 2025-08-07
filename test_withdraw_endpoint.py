#!/usr/bin/env python3

import requests
import json

# API base URL
BASE_URL = "http://localhost:8536/api/v4"

def test_withdraw_endpoint():
    """Test the withdraw endpoint functionality"""
    
    print("=== Testing Withdraw Endpoint ===")
    
    # These are the JWTs from our previous tests
    EMPLOYER_JWT = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxNSIsImlzcyI6ImZhc3Rqb2JfZGV2IiwiaWF0IjoxNzU0MjMyMjc0LCJleHAiOjE3NTQyNzU0NzQsInNlc3Npb24iOiI2M2NhOTI1OGI1NDQ0MDEzOTQ1N2IzMGQ1ODk3Y2FiMyIsInJvbGUiOiJFbXBsb3llciIsImVtYWlsIjoidGVzdGVtcGxveWVyMjAyNUBleGFtcGxlLmNvbSIsImxhbmciOiJicm93c2VyIn0.Lu0ZgtTJ8S8UDQ3dpaa-v6ZUwJb9wJh4LsQvClgW5hc"
    
    headers = {
        "Authorization": f"Bearer {EMPLOYER_JWT}",
        "Content-Type": "application/json"
    }
    
    # Step 1: Check current wallet balance
    print("1. Checking current wallet balance...")
    wallet_response = requests.get(f"{BASE_URL}/account/wallet", headers=headers)
    
    if wallet_response.status_code == 200:
        wallet_data = wallet_response.json()
        current_balance = float(wallet_data.get('available_balance', 0))
        print(f"   Current balance: ${current_balance}")
        
        if current_balance <= 0:
            print("   ❌ Insufficient balance for withdraw test")
            return
        
        # Step 2: Test withdraw with small amount
        withdraw_amount = min(10.0, current_balance - 5.0)  # Leave some balance
        print(f"2. Testing withdraw of ${withdraw_amount}...")
        
        withdraw_data = {
            "amount": str(withdraw_amount)
        }
        
        withdraw_response = requests.post(
            f"{BASE_URL}/account/wallet/withdraw", 
            headers=headers, 
            json=withdraw_data
        )
        
        print(f"   Status code: {withdraw_response.status_code}")
        
        if withdraw_response.status_code == 200:
            result = withdraw_response.json()
            print("   ✅ Withdraw successful!")
            print(f"   Previous balance: ${result.get('previous_balance', 'N/A')}")
            print(f"   New balance: ${result.get('new_balance', 'N/A')}")
            print(f"   Transaction ID: {result.get('transaction_id', 'N/A')}")
            
            # Step 3: Verify wallet balance changed
            print("3. Verifying wallet balance updated...")
            new_wallet_response = requests.get(f"{BASE_URL}/account/wallet", headers=headers)
            if new_wallet_response.status_code == 200:
                new_wallet_data = new_wallet_response.json()
                new_balance = float(new_wallet_data.get('available_balance', 0))
                print(f"   New balance: ${new_balance}")
                
                expected_balance = current_balance - withdraw_amount
                if abs(new_balance - expected_balance) < 0.01:  # Allow for floating point precision
                    print("   ✅ Balance updated correctly!")
                else:
                    print(f"   ❌ Balance mismatch. Expected: ${expected_balance}, Got: ${new_balance}")
            else:
                print(f"   ❌ Failed to check updated balance: {new_wallet_response.text}")
        else:
            print("   ❌ Withdraw failed!")
            print(f"   Response: {withdraw_response.text}")
    else:
        print(f"   ❌ Failed to get wallet info: {wallet_response.text}")

if __name__ == "__main__":
    try:
        test_withdraw_endpoint()
    except Exception as e:
        print(f"❌ Test failed with error: {e}")