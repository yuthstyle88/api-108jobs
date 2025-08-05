# Freelancer Payment System Testing Guide

## Overview
This guide covers testing the three-phase freelancer payment workflow:
1. **Quotation** → Freelancer creates detailed proposal
2. **Order** → Employer approves quotation 
3. **Invoice** → Ready for payment processing

**Server URL**: `http://localhost:8536`

---

## Prerequisites

### Required Tools
- **curl** or **Postman** for API testing
- **Database client** (optional, for verification)
- **JWT token** for authentication

### Server Status
```bash
# Verify server is running
curl http://localhost:8536/api/v4/site

# Expected: Site information response
```

---

## Test Flow 1: User Setup and Wallet Creation

### Step 1.1: Register Freelancer Account
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "freelancer_test",
    "password": "test123456",
    "password_verify": "test123456",
    "email": "freelancer@test.com",
    "show_nsfw": false,
    "captcha_uuid": null,
    "captcha_answer": null,
    "honeypot": null,
    "answer": null
  }'
```

### Step 1.2: Register Employer Account  
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "employer_test", 
    "password": "test123456",
    "password_verify": "test123456",
    "email": "employer@test.com",
    "show_nsfw": false,
    "captcha_uuid": null,
    "captcha_answer": null,
    "honeypot": null,
    "answer": null
  }'
```

### Step 1.3: Login Freelancer
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "freelancer_test",
    "password": "test123456"
  }'
```
**Save the JWT token from response for FREELANCER_JWT**

### Step 1.4: Login Employer
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "employer_test", 
    "password": "test123456"
  }'
```
**Save the JWT token from response for EMPLOYER_JWT**

### Step 1.5: Verify Automatic Wallet Creation
```bash
# Check freelancer wallet
curl -X GET http://localhost:8536/api/v4/account/wallet \
  -H "Authorization: Bearer FREELANCER_JWT"

# Check employer wallet  
curl -X GET http://localhost:8536/api/v4/account/wallet \
  -H "Authorization: Bearer EMPLOYER_JWT"

# Expected: Both should have wallets with balance: 0.0, escrow_balance: 0.0
```

---

## Test Flow 2: Post Creation (Required for Job Context)

### Step 2.1: Create a Job Post (as Employer)
```bash
curl -X POST http://localhost:8536/api/v4/post \
  -H "Authorization: Bearer EMPLOYER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Website Development Project",
    "body": "Need a freelancer to build a modern website with React and Node.js",
    "community_id": 1,
    "nsfw": false
  }'
```
**Save the post_id from response for later use**

---

## Test Flow 3: Phase 1 - Quotation Creation

### Step 3.1: Freelancer Creates Detailed Quotation
```bash
curl -X POST http://localhost:8536/api/v4/account/wallet/create-invoice \
  -H "Authorization: Bearer FREELANCER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "employer_id": EMPLOYER_USER_ID,
    "post_id": POST_ID,
    "comment_id": null,
    "price": 2500.00,
    "proposal": "I will create a modern, responsive website using React for frontend and Node.js for backend. The site will include user authentication, database integration, and deployment setup.",
    "name": "Modern Website Development",
    "job_description": "Full-stack web development including frontend, backend, database design, and deployment",
    "work_steps": [
      "Requirements analysis and planning",
      "Database schema design", 
      "Backend API development with Node.js",
      "Frontend development with React",
      "Integration and testing",
      "Deployment and documentation"
    ],
    "revise_times": 3,
    "revise_description": "Up to 3 rounds of revisions included for design changes, content updates, or minor functionality adjustments",
    "working_days": 21,
    "deliverables": [
      "Complete source code (frontend + backend)",
      "Deployed application on cloud platform", 
      "Database setup and configuration",
      "API documentation",
      "User manual and admin guide"
    ],
    "note": "Price includes hosting setup for first 3 months. Additional features can be discussed separately.",
    "starting_day": "2025-08-05",
    "delivery_day": "2025-08-26"
  }'
```

### Step 3.2: Verify Quotation Status
**Expected Response:**
- `status: "QuotationPending"`
- `billing_id: [some_id]` 
- `success: true`

**Save the billing_id for next steps**

---

## Test Flow 4: Phase 2 - Order Approval

### Step 4.1: Employer Reviews and Approves Quotation
```bash
curl -X POST http://localhost:8536/api/v4/account/wallet/approve-quotation \
  -H "Authorization: Bearer EMPLOYER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "billing_id": BILLING_ID
  }'
```

### Step 4.2: Verify Status Change
**Expected Response:**
- `status: "OrderApproved"`
- `billing_id: [same_id]`
- `success: true`

---

## Test Flow 5: Admin Wallet Operations

### Step 5.1: Create Admin User (if needed)
```bash
# Register admin
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin_test",
    "password": "admin123456", 
    "password_verify": "admin123456",
    "email": "admin@test.com",
    "show_nsfw": false
  }'

# Login admin
curl -X POST http://localhost:8536/api/v4/account/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "admin_test",
    "password": "admin123456"
  }'
```

### Step 5.2: Admin Top-up Employer Wallet
```bash
curl -X POST http://localhost:8536/api/v4/admin/wallet/top-up \
  -H "Authorization: Bearer ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": EMPLOYER_USER_ID,
    "amount": 5000.00,
    "reason": "Initial wallet funding for testing"
  }'
```

### Step 5.3: Verify Wallet Balance
```bash
curl -X GET http://localhost:8536/api/v4/account/wallet \
  -H "Authorization: Bearer EMPLOYER_JWT"

# Expected: balance should now be 5000.00
```

---

## Test Flow 6: Error Handling and Edge Cases

### Step 6.1: Test Invalid Quotation Data
```bash
# Test negative price
curl -X POST http://localhost:8536/api/v4/account/wallet/create-invoice \
  -H "Authorization: Bearer FREELANCER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "employer_id": EMPLOYER_USER_ID,
    "post_id": POST_ID,
    "price": -100.00,
    "proposal": "Test proposal",
    "name": "Test",
    "job_description": "Test job",
    "work_steps": ["Step 1"],
    "revise_times": 1,
    "revise_description": "Test",
    "working_days": 5,
    "deliverables": ["Test deliverable"],
    "starting_day": "2025-08-05",
    "delivery_day": "2025-08-10"
  }'

# Expected: Error about price must be positive
```

### Step 6.2: Test Unauthorized Approval
```bash
# Try to approve with freelancer token (should fail)
curl -X POST http://localhost:8536/api/v4/account/wallet/approve-quotation \
  -H "Authorization: Bearer FREELANCER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "billing_id": BILLING_ID
  }'

# Expected: Error - only employer can approve their quotations
```

### Step 6.3: Test Double Approval
```bash
# Try to approve same quotation twice
curl -X POST http://localhost:8536/api/v4/account/wallet/approve-quotation \
  -H "Authorization: Bearer EMPLOYER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "billing_id": BILLING_ID
  }'

# Expected: Error - quotation already approved
```

---

## Test Flow 7: Account Integration Testing

### Step 7.1: Verify Wallet Info in Account Response
```bash
curl -X GET http://localhost:8536/api/v4/account \
  -H "Authorization: Bearer FREELANCER_JWT"

# Expected: Response should include wallet information
```

---

## Verification Checklist

### ✅ Basic Functionality
- [ ] User registration creates wallet automatically
- [ ] Freelancer can create detailed quotations
- [ ] Employer can approve quotations
- [ ] Admin can manage wallets
- [ ] Account endpoint includes wallet info

### ✅ Business Logic
- [ ] Quotation status: `QuotationPending` → `OrderApproved`
- [ ] Only employer can approve their quotations
- [ ] Price validation (must be positive)
- [ ] Working days validation (must be positive)
- [ ] Revise times validation (cannot be negative)

### ✅ Security
- [ ] JWT authentication required for all endpoints
- [ ] Admin endpoints require admin privileges
- [ ] Users can only access their own wallet data
- [ ] Proper authorization for quotation approval

### ✅ Data Integrity
- [ ] All quotation fields saved correctly
- [ ] Detailed description formatting works
- [ ] Status changes tracked properly
- [ ] Foreign key relationships maintained

---

## Common Issues and Solutions

### Issue: "User not found"
**Solution**: Ensure you're using the correct user_id from the registration response

### Issue: "Billing not found" 
**Solution**: Verify the billing_id from the quotation creation response

### Issue: "Authentication failed"
**Solution**: Check JWT token is valid and properly formatted in Authorization header

### Issue: "Permission denied"
**Solution**: Ensure user has correct role (admin for admin endpoints, employer for approvals)

---

## Next Steps After Testing

Once basic testing is complete, you can extend with:
1. Payment processing integration
2. Work submission workflow  
3. Revision request handling
4. Dispute resolution system
5. Notification system for status changes

---

**Happy Testing! 🚀**