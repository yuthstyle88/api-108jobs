# Bank Account System Testing Guide

This guide walks you through testing the complete region-based bank account system.

## Prerequisites

1. **Start the server**:
   ```bash
   cargo run
   # Server should be running on http://localhost:8536
   ```

2. **Ensure database is set up**:
   ```bash
   # Run any pending migrations
   cargo run -- migration run
   ```

## Quick Test (Automated)

### Method 1: Python Script (Recommended)

```bash
# Make the script executable
chmod +x test_bank_flow.py

# Run the test
python3 test_bank_flow.py http://localhost:8536 testuser123 password123 test@example.com
```

The script will automatically:
- ✅ Register a new user with IP-based country detection
- ✅ Get user info to verify detected country
- ✅ List banks filtered by user's region
- ✅ Create a bank account with verification image
- ✅ List user's bank accounts with verification status

## Manual Testing Steps

### Step 1: Register New User
```bash
curl -X POST http://localhost:8536/api/v3/user/register \
  -H "Content-Type: application/json" \
  -H "User-Agent: TestClient/1.0" \
  -d '{
    "username": "testuser123",
    "password": "password123", 
    "password_verify": "password123",
    "email": "test@example.com",
    "show_nsfw": false,
    "captcha_uuid": "",
    "captcha_answer": "",
    "honeypot": ""
  }'
```

**Expected Response:**
```json
{
  "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "registration_created": false,
  "verify_email_sent": false
}
```

**🔑 Save the JWT token for subsequent requests.**

### Step 2: Check User's Detected Country
```bash
curl -X GET http://localhost:8536/api/v3/user \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

**Expected Response:**
```json
{
  "local_user_view": {
    "local_user": {
      "id": 1,
      "country": "Thailand",
      ...
    }
  }
}
```

### Step 3: List Banks (Filtered by Country)
```bash
curl -X GET http://localhost:8536/api/v1/banks \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

**Expected Response:**
```json
{
  "banks": [
    {
      "id": 1,
      "name": "Bangkok Bank",
      "country": "Thailand",
      "bank_code": "BBL",
      "swift_code": "BKKBTHBK"
    },
    {
      "id": 2,
      "name": "Kasikorn Bank", 
      "country": "Thailand",
      "bank_code": "KBANK",
      "swift_code": "KASITHBK"
    }
  ]
}
```

### Step 4: Create Bank Account
```bash
curl -X POST http://localhost:8536/api/v1/user/bank_account/create \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "bank_id": 1,
    "account_number": "1234567890",
    "account_name": "Test User Account",
    "is_default": true,
    "verification_image": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAAAAAAAD..."
  }'
```

**Expected Response:**
```json
{
  "bank_account_id": 1,
  "success": true
}
```

### Step 5: List User Bank Accounts
```bash
curl -X GET http://localhost:8536/api/v1/user/bank_accounts \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

**Expected Response:**
```json
{
  "bank_accounts": [
    {
      "id": 1,
      "bank_id": 1,
      "bank_name": "Bangkok Bank",
      "bank_country": "Thailand",
      "account_number": "1234567890",
      "account_name": "Test User Account",
      "is_default": true,
      "is_verified": false,
      "created_at": "2025-08-05T00:00:00Z"
    }
  ]
}
```

## Testing Different Scenarios

### Test Region Restrictions

**Thailand User (should work):**
```bash
python3 test_bank_flow.py http://localhost:8536 thai_user password123 thai@example.com
```

**Try to add Vietnam bank as Thailand user (should fail):**
```bash
curl -X POST http://localhost:8536/api/v1/user/bank_account/create \
  -H "Authorization: Bearer THAILAND_USER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "bank_id": 5,
    "account_number": "9876543210",
    "account_name": "Test Account",
    "is_default": true
  }'
```

**Expected Error:**
```json
{
  "error": "Bank Vietcombank is not available in your region (Thailand)"
}
```

## Admin Testing

### List Unverified Bank Accounts (Admin Only)
```bash
curl -X GET http://localhost:8536/api/v1/admin/bank_accounts/unverified \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

### Verify Bank Account (Admin Only)
```bash
curl -X POST http://localhost:8536/api/v1/admin/bank_accounts/verify \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "bank_account_id": 1,
    "verified": true,
    "admin_notes": "Bank statement verified"
  }'
```

## Expected Behaviors

### ✅ What Should Work

1. **IP-based Country Detection**: Users get their country auto-detected during registration
2. **Region-based Bank Filtering**: Users only see banks from their country  
3. **Cross-region Validation**: Users cannot add banks from other countries
4. **Verification Workflow**: Bank accounts start unverified, need admin approval
5. **Default Account Management**: First account becomes default automatically

### 🚫 What Should Fail

1. **Cross-region Bank Account**: Thailand user trying to add Vietnam bank
2. **Non-admin Verification**: Regular user trying to verify accounts
3. **Invalid Bank ID**: Using non-existent bank ID
4. **Missing Required Fields**: Account creation without required data

## Troubleshooting

### Common Issues

**"Country not detected"**: 
- Check if IP geolocation service is accessible
- Default should be Thailand for local development

**"No banks available"**:
- Ensure banks are seeded in database
- Check if banks exist for user's detected country

**"Bank not available in region"**:
- This is expected when testing cross-region restrictions
- Verify user's country matches bank's country

**JWT Token Issues**:
- Ensure token is included in Authorization header: `Bearer YOUR_TOKEN`
- Check token hasn't expired

### Debug Commands

```bash
# Check user's country in database
psql -d your_db -c "SELECT id, person_id, country FROM local_user WHERE id = 1;"

# Check available banks
psql -d your_db -c "SELECT id, name, country, is_active FROM banks;"

# Check user bank accounts
psql -d your_db -c "SELECT * FROM user_bank_accounts WHERE user_id = 1;"
```

## Testing Checklist

- [ ] User registration with automatic country detection
- [ ] Banks filtered by user's country
- [ ] Bank account creation with verification image
- [ ] Cross-region bank account creation blocked
- [ ] Admin can list unverified accounts
- [ ] Admin can verify/reject bank accounts
- [ ] Default bank account management works
- [ ] Verification status displayed correctly

## Quick Start

1. **Start server**: `cargo run`
2. **Run test**: `python3 test_bank_flow.py http://localhost:8536`
3. **Check results**: The script will show you each step's success/failure

This system ensures users only see and can use banks appropriate for their region while maintaining proper verification workflows for financial security! 🏦