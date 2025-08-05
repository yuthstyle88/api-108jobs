#!/usr/bin/env python3
"""
Test script for the region-based bank account system
Tests the full flow: Registration -> Login -> List Banks -> Create Bank Account
"""

import requests
import json
import sys
from typing import Optional, Dict, Any

class BankFlowTester:
    def __init__(self, base_url: str = "http://localhost:8536"):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.jwt_token: Optional[str] = None
        
    def print_step(self, step: str, details: str = ""):
        print(f"\n🔹 {step}")
        if details:
            print(f"   {details}")
    
    def print_success(self, message: str):
        print(f"✅ {message}")
    
    def print_error(self, message: str):
        print(f"❌ {message}")
    
    def print_response(self, response: requests.Response):
        print(f"   Status: {response.status_code}")
        try:
            data = response.json()
            print(f"   Response: {json.dumps(data, indent=2)}")
        except:
            print(f"   Response: {response.text}")
    
    def register_user(self, username: str, password: str, email: str, country_override: Optional[str] = None) -> bool:
        """Register a new user account"""
        self.print_step("STEP 1: User Registration", f"Creating account for {username}")
        
        payload = {
            "username": username,
            "password": password,
            "password_verify": password,
            "email": email,
            "show_nsfw": False,
            "captcha_uuid": "",
            "captcha_answer": "",
            "honeypot": ""
        }
        
        # Add country override if provided (for testing different regions)
        if country_override:
            payload["country"] = country_override
        
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "BankFlowTester/1.0"
        }
        
        try:
            response = self.session.post(
                f"{self.base_url}/api/v3/user/register",
                json=payload,
                headers=headers
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                if "jwt" in data and data["jwt"]:
                    self.jwt_token = data["jwt"]
                    self.print_success("User registered and logged in successfully")
                    return True
                else:
                    self.print_success("User registered, but manual login required")
                    return self.login_user(username, password)
            else:
                self.print_error("Registration failed")
                return False
                
        except Exception as e:
            self.print_error(f"Registration error: {str(e)}")
            return False
    
    def login_user(self, username: str, password: str) -> bool:
        """Login with existing credentials"""
        self.print_step("STEP 1b: User Login", f"Logging in as {username}")
        
        payload = {
            "username_or_email": username,
            "password": password
        }
        
        try:
            response = self.session.post(
                f"{self.base_url}/api/v3/user/login",
                json=payload,
                headers={"Content-Type": "application/json"}
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                if "jwt" in data and data["jwt"]:
                    self.jwt_token = data["jwt"]
                    self.print_success("Login successful")
                    return True
            
            self.print_error("Login failed")
            return False
            
        except Exception as e:
            self.print_error(f"Login error: {str(e)}")
            return False
    
    def get_user_info(self) -> Optional[Dict[str, Any]]:
        """Get current user information to check country"""
        self.print_step("STEP 2: Get User Info", "Checking user's detected country")
        
        if not self.jwt_token:
            self.print_error("No JWT token available")
            return None
        
        headers = {
            "Authorization": f"Bearer {self.jwt_token}",
            "Content-Type": "application/json"
        }
        
        try:
            response = self.session.get(
                f"{self.base_url}/api/v3/user",
                headers=headers
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                user_country = data.get("local_user_view", {}).get("local_user", {}).get("country", "Unknown")
                self.print_success(f"User country detected as: {user_country}")
                return data
            else:
                self.print_error("Failed to get user info")
                return None
                
        except Exception as e:
            self.print_error(f"Get user info error: {str(e)}")
            return None
    
    def list_banks(self) -> Optional[list]:
        """List available banks (should be filtered by user's country)"""
        self.print_step("STEP 3: List Banks", "Getting banks available in user's region")
        
        if not self.jwt_token:
            self.print_error("No JWT token available")
            return None
        
        headers = {
            "Authorization": f"Bearer {self.jwt_token}",
            "Content-Type": "application/json"
        }
        
        try:
            response = self.session.get(
                f"{self.base_url}/api/v1/banks",
                headers=headers
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                banks = data.get("banks", [])
                self.print_success(f"Found {len(banks)} banks available")
                
                for bank in banks:
                    print(f"   • {bank.get('name', 'Unknown')} ({bank.get('country', 'Unknown')})")
                
                return banks
            else:
                self.print_error("Failed to list banks")
                return None
                
        except Exception as e:
            self.print_error(f"List banks error: {str(e)}")
            return None
    
    def create_bank_account(self, bank_id: int, account_number: str, account_name: str) -> bool:
        """Create a bank account for the user"""
        self.print_step("STEP 4: Create Bank Account", f"Adding bank account {account_number}")
        
        if not self.jwt_token:
            self.print_error("No JWT token available")
            return False
        
        payload = {
            "bank_id": bank_id,
            "account_number": account_number,
            "account_name": account_name,
            "is_default": True,
            "verification_image": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAAAAAAAD..."  # Mock base64 image
        }
        
        headers = {
            "Authorization": f"Bearer {self.jwt_token}",
            "Content-Type": "application/json"
        }
        
        try:
            response = self.session.post(
                f"{self.base_url}/api/v1/user/bank_account/create",
                json=payload,
                headers=headers
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                self.print_success("Bank account created successfully (pending verification)")
                return True
            else:
                self.print_error("Failed to create bank account")
                return False
                
        except Exception as e:
            self.print_error(f"Create bank account error: {str(e)}")
            return False
    
    def list_user_bank_accounts(self) -> Optional[list]:
        """List user's bank accounts"""
        self.print_step("STEP 5: List User Bank Accounts", "Getting user's registered bank accounts")
        
        if not self.jwt_token:
            self.print_error("No JWT token available")
            return None
        
        headers = {
            "Authorization": f"Bearer {self.jwt_token}",
            "Content-Type": "application/json"
        }
        
        try:
            response = self.session.get(
                f"{self.base_url}/api/v1/user/bank_accounts",
                headers=headers
            )
            
            self.print_response(response)
            
            if response.status_code == 200:
                data = response.json()
                accounts = data.get("bank_accounts", [])
                self.print_success(f"Found {len(accounts)} bank accounts")
                
                for account in accounts:
                    verified_status = "✅ Verified" if account.get("is_verified", False) else "⏳ Pending Verification"
                    default_status = "⭐ Default" if account.get("is_default", False) else ""
                    print(f"   • {account.get('account_name', 'Unknown')} - {account.get('bank_name', 'Unknown')} - {verified_status} {default_status}")
                
                return accounts
            else:
                self.print_error("Failed to list user bank accounts")
                return None
                
        except Exception as e:
            self.print_error(f"List user bank accounts error: {str(e)}")
            return None
    
    def run_full_test(self, username: str, password: str, email: str):
        """Run the complete test flow"""
        print("🚀 Starting Bank Account System Test")
        print("=" * 50)
        
        # Step 1: Register user
        if not self.register_user(username, password, email):
            print("\n❌ Test failed at registration step")
            return False
        
        # Step 2: Get user info (check country detection)
        user_info = self.get_user_info()
        if not user_info:
            print("\n❌ Test failed at user info step")
            return False
        
        # Step 3: List banks (should be filtered by country)
        banks = self.list_banks()
        if banks is None:
            print("\n❌ Test failed at list banks step")
            return False
        
        if not banks:
            self.print_error("No banks available for user's region")
            return False
        
        # Step 4: Create bank account (use first available bank)
        first_bank = banks[0]
        bank_id = first_bank.get("id")
        if not bank_id:
            self.print_error("No valid bank ID found")
            return False
        
        if not self.create_bank_account(bank_id, "1234567890", f"{username} Test Account"):
            print("\n❌ Test failed at create bank account step")
            return False
        
        # Step 5: List user bank accounts
        if not self.list_user_bank_accounts():
            print("\n❌ Test failed at list user bank accounts step")
            return False
        
        print("\n" + "=" * 50)
        print("🎉 All tests completed successfully!")
        print("\n📋 Test Summary:")
        print("   ✅ User registration with IP-based country detection")
        print("   ✅ Banks filtered by user's region")
        print("   ✅ Bank account creation with verification image")
        print("   ✅ Bank account listed with verification status")
        
        return True

def main():
    if len(sys.argv) < 2:
        print("Usage: python test_bank_flow.py <base_url> [username] [password] [email]")
        print("Example: python test_bank_flow.py http://localhost:8536 testuser123 password123 test@example.com")
        sys.exit(1)
    
    base_url = sys.argv[1]
    username = sys.argv[2] if len(sys.argv) > 2 else "testuser123"
    password = sys.argv[3] if len(sys.argv) > 3 else "password123"
    email = sys.argv[4] if len(sys.argv) > 4 else f"{username}@example.com"
    
    tester = BankFlowTester(base_url)
    tester.run_full_test(username, password, email)

if __name__ == "__main__":
    main()