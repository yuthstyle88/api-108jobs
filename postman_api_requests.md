# FastWork API Requests for Postman

## Complete Freelancer Workflow Test

### 1. Get Site Info
```
GET http://localhost:8536/api/v4/site
```

### 2. Register Employer
**IMPORTANT**: Use camelCase field names and get captcha first!

#### 2a. Get Captcha
```
GET http://localhost:8536/api/v4/account/auth/get-captcha
```
**Save the UUID from response**

#### 2b. Register with camelCase fields
```
POST http://localhost:8536/api/v4/account/auth/register
Content-Type: application/json

{
  "username": "employercamel123",
  "password": "testpassword123",
  "passwordVerify": "testpassword123",
  "email": "employer@example.com",
  "showNsfw": false,
  "captchaUuid": "[UUID_FROM_CAPTCHA_RESPONSE]",
  "captchaAnswer": "test",
  "honeypot": ""
}
```

### 3. Register Freelancer
```
POST http://localhost:8536/api/v4/account/auth/register
Content-Type: application/json

{
  "username": "freelancercamel123",
  "password": "testpassword123",
  "password_verify": "testpassword123",
  "email": "freelancer@example.com",
  "show_nsfw": false,
  "captcha_uuid": "",
  "captcha_answer": "",
  "honeypot": ""
}
```

### 4. Login as Employer
```
POST http://localhost:8536/api/v4/account/auth/login
Content-Type: application/json

{
  "username_or_email": "employercamel123",
  "password": "testpassword123"
}
```
**Save the JWT token from response for employer requests**

### 5. Login as Freelancer
```
POST http://localhost:8536/api/v4/account/auth/login
Content-Type: application/json

{
  "username_or_email": "freelancercamel123",
  "password": "testpassword123"
}
```
**Save the JWT token from response for freelancer requests**

### 6. Get Employer Wallet (Initial Check)
```
GET http://localhost:8536/api/v4/account/wallet
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
```

### 7. Get Freelancer Wallet (Initial Check)
```
GET http://localhost:8536/api/v4/account/wallet
Authorization: Bearer [FREELANCER_JWT_TOKEN]
```

### 8. Admin Top-up Employer Wallet
```
POST http://localhost:8536/api/v4/admin/wallet/top-up
Authorization: Bearer [ADMIN_JWT_TOKEN]
Content-Type: application/json

{
  "userId": [EMPLOYER_USER_ID],
  "amount": 1000.0,
  "reason": "Initial funding for testing"
}
```

### 9. Freelancer Creates Invoice/Quotation
```
POST http://localhost:8536/api/v4/account/services/create-invoice
Authorization: Bearer [FREELANCER_JWT_TOKEN]
Content-Type: application/json

{
  "employerId": [EMPLOYER_USER_ID],
  "postId": 1,
  "commentId": null,
  "price": 250.0,
  "proposal": "I will create a modern, responsive website for your business with the latest technologies.",
  "name": "Business Website Development",
  "jobDescription": "Create a professional website with modern design, responsive layout, and SEO optimization.",
  "workSteps": [
    "Initial consultation and requirements gathering",
    "Design mockups and wireframes",
    "Frontend development with React",
    "Backend API development",
    "Testing and deployment",
    "Final review and handover"
  ],
  "reviseTimes": 3,
  "reviseDescription": "Up to 3 rounds of revisions included for design and functionality changes",
  "workingDays": 14,
  "deliverables": [
    "Fully functional website",
    "Source code repository",
    "Documentation",
    "Deployment guide"
  ],
  "note": "Available for communication during business hours EST",
  "startingDay": "2025-08-05",
  "deliveryDay": "2025-08-19"
}
```

### 10. Employer Approves Quotation
```
POST http://localhost:8536/api/v4/account/services/approve-quotation
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
Content-Type: application/json

{
  "billingId": [BILLING_ID_FROM_STEP_9]
}
```

### 11. Check Employer Wallet After Payment
```
GET http://localhost:8536/api/v4/account/wallet
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
```

### 12. Freelancer Submits Work
```
POST http://localhost:8536/api/v4/account/services/submit-work
Authorization: Bearer [FREELANCER_JWT_TOKEN]
Content-Type: application/json

{
  "billingId": [BILLING_ID],
  "workDescription": "Completed website development with all requested features. The website is fully responsive and includes SEO optimization.",
  "deliverableUrl": "https://github.com/freelancer/business-website"
}
```

### 13. Employer Requests Revision
```
POST http://localhost:8536/api/v4/account/services/request-revision
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
Content-Type: application/json

{
  "billingId": [BILLING_ID],
  "revisionFeedback": "Please update the color scheme to match our brand colors and add a contact form to the main page."
}
```

### 14. Freelancer Re-submits Work After Revision
```
POST http://localhost:8536/api/v4/account/services/submit-work
Authorization: Bearer [FREELANCER_JWT_TOKEN]
Content-Type: application/json

{
  "billingId": [BILLING_ID],
  "workDescription": "Updated website with revised color scheme matching brand guidelines and added contact form as requested.",
  "deliverableUrl": "https://github.com/freelancer/business-website-v2"
}
```

### 15. Employer Approves Final Work
```
POST http://localhost:8536/api/v4/account/services/approve-work
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
Content-Type: application/json

{
  "billingId": [BILLING_ID]
}
```

### 16. Check Final Wallet Balances

#### Employer Wallet (Final)
```
GET http://localhost:8536/api/v4/account/wallet
Authorization: Bearer [EMPLOYER_JWT_TOKEN]
```

#### Freelancer Wallet (Final)
```
GET http://localhost:8536/api/v4/account/wallet
Authorization: Bearer [FREELANCER_JWT_TOKEN]
```

## Notes for Postman Setup

1. **Environment Variables**: Create variables for:
   - `base_url`: http://localhost:8536
   - `employer_token`: JWT token from employer login
   - `freelancer_token`: JWT token from freelancer login
   - `employer_user_id`: User ID from employer login response
   - `freelancer_user_id`: User ID from freelancer login response
   - `billing_id`: Billing ID from create-invoice response

2. **Headers**: Most requests need:
   - `Content-Type: application/json`
   - `Authorization: Bearer {{token_variable}}`

3. **Request Order**: Execute requests in numerical order for proper workflow

4. **Token Management**: JWT tokens expire after 1-2 hours, re-login if needed

5. **API Changes**: Note the new `/services` scope instead of `/billing` or `/wallet` for service operations