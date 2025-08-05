#!/bin/bash

# FastWork API Workflow Test Script
BASE_URL="http://localhost:8536"

echo "=== Testing Complete Freelancer Workflow with /services endpoints ==="

# Step 1: Test site availability
echo "1. Testing site availability..."
SITE_RESPONSE=$(curl -s "$BASE_URL/api/v4/site")
if [[ $SITE_RESPONSE == *"siteView"* ]]; then
    echo "✅ Site is available"
else
    echo "❌ Site not available"
    exit 1
fi

# Step 2: Try to login with existing users first
echo -e "\n2. Attempting login with existing users..."

# Try employer login
EMPLOYER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username_or_email":"employercamel123","password":"testpassword123"}')

if [[ $EMPLOYER_RESPONSE == *"jwt"* ]]; then
    EMPLOYER_JWT=$(echo $EMPLOYER_RESPONSE | jq -r '.jwt')
    EMPLOYER_USER_ID=$(echo $EMPLOYER_RESPONSE | jq -r '.localUserView.localUser.id')
    echo "✅ Employer login successful - User ID: $EMPLOYER_USER_ID"
else
    echo "⚠️  Employer login failed: $EMPLOYER_RESPONSE"
    echo "Trying to register new employer..."
    
    # Register new employer
    REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/auth/register" \
        -H "Content-Type: application/json" \
        -d '{
            "username": "testemployer2025",
            "password": "SecurePass123!",
            "password_verify": "SecurePass123!",
            "email": "testemployer2025@example.com",
            "show_nsfw": false,
            "captcha_uuid": "",
            "captcha_answer": "",
            "honeypot": ""
        }')
    
    if [[ $REGISTER_RESPONSE == *"jwt"* ]]; then
        EMPLOYER_JWT=$(echo $REGISTER_RESPONSE | jq -r '.jwt')
        EMPLOYER_USER_ID=$(echo $REGISTER_RESPONSE | jq -r '.localUserView.localUser.id')
        echo "✅ New employer registered - User ID: $EMPLOYER_USER_ID"
    else
        echo "❌ Employer registration failed: $REGISTER_RESPONSE"
        exit 1
    fi
fi

# Try freelancer login
FREELANCER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username_or_email":"freelancercamel123","password":"testpassword123"}')

if [[ $FREELANCER_RESPONSE == *"jwt"* ]]; then
    FREELANCER_JWT=$(echo $FREELANCER_RESPONSE | jq -r '.jwt')
    FREELANCER_USER_ID=$(echo $FREELANCER_RESPONSE | jq -r '.localUserView.localUser.id')
    echo "✅ Freelancer login successful - User ID: $FREELANCER_USER_ID"
else
    echo "⚠️  Freelancer login failed: $FREELANCER_RESPONSE"
    echo "Trying to register new freelancer..."
    
    # Register new freelancer
    REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/auth/register" \
        -H "Content-Type: application/json" \
        -d '{
            "username": "testfreelancer2025",
            "password": "SecurePass123!",
            "password_verify": "SecurePass123!",
            "email": "testfreelancer2025@example.com",
            "show_nsfw": false,
            "captcha_uuid": "",
            "captcha_answer": "",
            "honeypot": ""
        }')
    
    if [[ $REGISTER_RESPONSE == *"jwt"* ]]; then
        FREELANCER_JWT=$(echo $REGISTER_RESPONSE | jq -r '.jwt')
        FREELANCER_USER_ID=$(echo $REGISTER_RESPONSE | jq -r '.localUserView.localUser.id')
        echo "✅ New freelancer registered - User ID: $FREELANCER_USER_ID"
    else
        echo "❌ Freelancer registration failed: $REGISTER_RESPONSE"
        exit 1
    fi
fi

# Step 3: Check initial wallet balances
echo -e "\n3. Checking initial wallet balances..."

EMPLOYER_WALLET=$(curl -s -X GET "$BASE_URL/api/v4/account/wallet" \
    -H "Authorization: Bearer $EMPLOYER_JWT")
echo "Employer wallet: $EMPLOYER_WALLET"

FREELANCER_WALLET=$(curl -s -X GET "$BASE_URL/api/v4/account/wallet" \
    -H "Authorization: Bearer $FREELANCER_JWT")
echo "Freelancer wallet: $FREELANCER_WALLET"

# Step 4: Create invoice using NEW /services endpoint
echo -e "\n4. Creating invoice via /services/create-invoice..."

INVOICE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/create-invoice" \
    -H "Authorization: Bearer $FREELANCER_JWT" \
    -H "Content-Type: application/json" \
    -d "{
        \"employerId\": $EMPLOYER_USER_ID,
        \"postId\": 1,
        \"commentId\": null,
        \"price\": 250.0,
        \"proposal\": \"I will create a modern responsive website\",
        \"name\": \"Website Development Project\",
        \"jobDescription\": \"Create a professional website with modern design\",
        \"workSteps\": [\"Design\", \"Development\", \"Testing\", \"Deployment\"],
        \"reviseTimes\": 3,
        \"reviseDescription\": \"Up to 3 revisions included\",
        \"workingDays\": 14,
        \"deliverables\": [\"Website\", \"Source code\", \"Documentation\"],
        \"note\": \"Available during business hours\",
        \"startingDay\": \"2025-08-05\",
        \"deliveryDay\": \"2025-08-19\"
    }")

if [[ $INVOICE_RESPONSE == *"billingId"* ]]; then
    BILLING_ID=$(echo $INVOICE_RESPONSE | jq -r '.billingId')
    echo "✅ Invoice created successfully - Billing ID: $BILLING_ID"
    echo "Full response: $INVOICE_RESPONSE"
else
    echo "❌ Invoice creation failed: $INVOICE_RESPONSE"
    exit 1
fi

# Step 5: Approve quotation using NEW /services endpoint
echo -e "\n5. Approving quotation via /services/approve-quotation..."

APPROVE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/approve-quotation" \
    -H "Authorization: Bearer $EMPLOYER_JWT" \
    -H "Content-Type: application/json" \
    -d "{\"billingId\": $BILLING_ID}")

if [[ $APPROVE_RESPONSE == *"success"* ]]; then
    echo "✅ Quotation approved successfully"
    echo "Response: $APPROVE_RESPONSE"
else
    echo "❌ Quotation approval failed: $APPROVE_RESPONSE"
fi

# Step 6: Submit work using NEW /services endpoint
echo -e "\n6. Submitting work via /services/submit-work..."

SUBMIT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/submit-work" \
    -H "Authorization: Bearer $FREELANCER_JWT" \
    -H "Content-Type: application/json" \
    -d "{
        \"billingId\": $BILLING_ID,
        \"workDescription\": \"Completed website with all features\",
        \"deliverableUrl\": \"https://github.com/example/project\"
    }")

if [[ $SUBMIT_RESPONSE == *"success"* ]]; then
    echo "✅ Work submitted successfully"
    echo "Response: $SUBMIT_RESPONSE"
else
    echo "❌ Work submission failed: $SUBMIT_RESPONSE"
fi

# Step 7: Request revision using NEW /services endpoint
echo -e "\n7. Requesting revision via /services/request-revision..."

REVISION_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/request-revision" \
    -H "Authorization: Bearer $EMPLOYER_JWT" \
    -H "Content-Type: application/json" \
    -d "{
        \"billingId\": $BILLING_ID,
        \"revisionFeedback\": \"Please update the color scheme and add contact form\"
    }")

if [[ $REVISION_RESPONSE == *"success"* ]]; then
    echo "✅ Revision requested successfully"
    echo "Response: $REVISION_RESPONSE"
else
    echo "❌ Revision request failed: $REVISION_RESPONSE"
fi

# Step 8: Re-submit work after revision
echo -e "\n8. Re-submitting work after revision..."

RESUBMIT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/submit-work" \
    -H "Authorization: Bearer $FREELANCER_JWT" \
    -H "Content-Type: application/json" \
    -d "{
        \"billingId\": $BILLING_ID,
        \"workDescription\": \"Updated website with revised color scheme and contact form\",
        \"deliverableUrl\": \"https://github.com/example/project-v2\"
    }")

if [[ $RESUBMIT_RESPONSE == *"success"* ]]; then
    echo "✅ Work re-submitted successfully"
    echo "Response: $RESUBMIT_RESPONSE"
else
    echo "❌ Work re-submission failed: $RESUBMIT_RESPONSE"
fi

# Step 9: Approve final work using NEW /services endpoint
echo -e "\n9. Approving final work via /services/approve-work..."

FINAL_APPROVE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v4/account/services/approve-work" \
    -H "Authorization: Bearer $EMPLOYER_JWT" \
    -H "Content-Type: application/json" \
    -d "{\"billingId\": $BILLING_ID}")

if [[ $FINAL_APPROVE_RESPONSE == *"success"* ]]; then
    echo "✅ Final work approved successfully"
    echo "Response: $FINAL_APPROVE_RESPONSE"
else
    echo "❌ Final work approval failed: $FINAL_APPROVE_RESPONSE"
fi

# Step 10: Check final wallet balances
echo -e "\n10. Checking final wallet balances..."

FINAL_EMPLOYER_WALLET=$(curl -s -X GET "$BASE_URL/api/v4/account/wallet" \
    -H "Authorization: Bearer $EMPLOYER_JWT")
echo "Final employer wallet: $FINAL_EMPLOYER_WALLET"

FINAL_FREELANCER_WALLET=$(curl -s -X GET "$BASE_URL/api/v4/account/wallet" \
    -H "Authorization: Bearer $FREELANCER_JWT")
echo "Final freelancer wallet: $FINAL_FREELANCER_WALLET"

echo -e "\n=== Workflow Test Complete ==="
echo "All endpoints tested with new /services scope structure!"