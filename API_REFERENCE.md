# Freelancer Payment System API Reference

## Base URL
```
http://localhost:8536
```

---

## 🔐 Authentication APIs

### 1. Register User
**POST** `/api/v4/account/auth/register`

**Headers:**
```json
{
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "username": "your_username",
  "password": "your_password", 
  "password_verify": "your_password",
  "email": "your_email@example.com",
  "show_nsfw": false,
  "captcha_uuid": null,
  "captcha_answer": null,
  "honeypot": null,
  "answer": null
}
```

**Response:**
```json
{
  "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "registration_created": true,
  "verify_email_sent": false
}
```

### 2. Login User
**POST** `/api/v4/account/auth/login`

**Headers:**
```json
{
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "username_or_email": "your_username",
  "password": "your_password"
}
```

**Response:**
```json
{
  "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "registration_created": false,
  "verify_email_sent": false
}
```

---

## 💼 Job/Post Management APIs

### 3. Create Job Post
**POST** `/api/v4/post`

**Headers:**
```json
{
  "Authorization": "Bearer YOUR_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "name": "Job Title Here",
  "body": "Detailed job description and requirements",
  "community_id": 1,
  "nsfw": false,
  "url": null,
  "honeypot": null,
  "language_id": null
}
```

**Response:**
```json
{
  "post_view": {
    "post": {
      "id": 123,
      "name": "Job Title Here",
      "body": "Detailed job description...",
      "creator_id": 456,
      "community_id": 1,
      "published": "2025-08-03T02:30:00Z",
      "updated": null,
      "deleted": false,
      "removed": false,
      "locked": false,
      "nsfw": false,
      "embed_title": null,
      "embed_description": null,
      "thumbnail_url": null,
      "ap_id": "http://localhost:8536/post/123",
      "local": true,
      "embed_video_url": null,
      "language_id": 0,
      "featured_community": false,
      "featured_local": false
    }
  }
}
```

---

## 💳 Wallet Management APIs

### 4. Get Wallet Information
**GET** `/api/v4/account/wallet`

**Headers:**
```json
{
  "Authorization": "Bearer YOUR_JWT_TOKEN"
}
```

**Response:**
```json
{
  "walletId": 1,
  "balance": 1000.50,
  "escrowBalance": 250.00
}
```

### 5. Deposit to Wallet
**POST** `/api/v4/account/wallet/deposit`

**Headers:**
```json
{
  "Authorization": "Bearer YOUR_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "amount": 500.00
}
```

**Response:**
```json
{
  "walletId": 1,
  "balance": 1500.50,
  "escrowBalance": 250.00,
  "transactionAmount": 500.00,
  "success": true
}
```

---

## 📋 Quotation/Billing APIs

### 6. Create Invoice/Quotation (Freelancer)
**POST** `/api/v4/account/wallet/create-invoice`

**Headers:**
```json
{
  "Authorization": "Bearer FREELANCER_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "employer_id": 456,
  "post_id": 123,
  "comment_id": null,
  "price": 2500.00,
  "proposal": "I will create a modern, responsive website using React for frontend and Node.js for backend. The site will include user authentication, database integration, and deployment setup.",
  "name": "Modern Website Development",
  "job_description": "Full-stack web development including frontend, backend, database design, and deployment",
  "work_steps": ["Requirements analysis and planning", "Database schema design", "Backend API development with Node.js", "Frontend development with React", "Integration and testing", "Deployment and documentation"],
  "revise_times": 3,
  "revise_description": "Up to 3 rounds of revisions included for design changes, content updates, or minor functionality adjustments",
  "working_days": 21,
  "deliverables": ["Complete source code (frontend + backend)", "Deployed application on cloud platform", "Database setup and configuration", "API documentation", "User manual and admin guide"],
  "note": "Price includes hosting setup for first 3 months. Additional features can be discussed separately.",
  "starting_day": "2025-08-05",
  "delivery_day": "2025-08-26"
}
```

**Response:**
```json
{
  "billingId": 789,
  "freelancerId": 123,
  "employerId": 456,
  "postId": 123,
  "amount": 2500.00,
  "status": "QuotationPending",
  "maxRevisions": 3,
  "deliveryTimeframeDays": 21,
  "createdAt": "2025-08-03T02:30:00Z",
  "success": true
}
```

### 7. Approve Quotation (Employer)
**POST** `/api/v4/account/wallet/approve-quotation`

**Headers:**
```json
{
  "Authorization": "Bearer EMPLOYER_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "billingId": 789
}
```

**Response:**
```json
{
  "billingId": 789,
  "status": "OrderApproved",
  "success": true
}
```

---

## 🔧 Admin APIs

### 8. Admin Top-up Wallet
**POST** `/api/v4/admin/wallet/top-up`

**Headers:**
```json
{
  "Authorization": "Bearer ADMIN_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "userId": 456,
  "amount": 1000.00,
  "reason": "Initial wallet funding for new user"
}
```

**Response:**
```json
{
  "userId": 456,
  "walletId": 2,
  "previousBalance": 100.00,
  "newBalance": 1100.00,
  "operationAmount": 1000.00,
  "reason": "Initial wallet funding for new user",
  "success": true
}
```

### 9. Admin Withdraw from Wallet
**POST** `/api/v4/admin/wallet/withdraw`

**Headers:**
```json
{
  "Authorization": "Bearer ADMIN_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "userId": 456,
  "amount": 200.00,
  "reason": "Administrative withdrawal for policy violation"
}
```

**Response:**
```json
{
  "userId": 456,
  "walletId": 2,
  "previousBalance": 1100.00,
  "newBalance": 900.00,
  "operationAmount": 200.00,
  "reason": "Administrative withdrawal for policy violation",
  "success": true
}
```

### 10. Add/Remove Admin (Existing Admin Only)
**POST** `/api/v4/admin/add`

**Description:** Only existing admins can promote users to admin or demote admins. The first admin is created automatically from the configuration file during initial setup.

**Headers:**
```json
{
  "Authorization": "Bearer ADMIN_JWT_TOKEN",
  "Content-Type": "application/json"
}
```

**Request Body:**
```json
{
  "person_id": 123,
  "added": true
}
```
- `person_id`: The ID of the person to promote/demote
- `added`: `true` to make admin, `false` to remove admin status

**Response:**
```json
{
  "admins": [
    {
      "person": {
        "id": 1,
        "name": "admin",
        "display_name": null,
        "avatar": null,
        "banned": false,
        "published": "2025-08-03T02:30:00Z",
        "updated": null,
        "actor_id": "http://localhost:8536/u/admin",
        "bio": null,
        "local": true,
        "banner": null,
        "deleted": false,
        "inbox_url": "http://localhost:8536/u/admin/inbox",
        "shared_inbox_url": null,
        "matrix_user_id": null,
        "admin": true,
        "bot_account": false,
        "ban_expires": null,
        "instance_id": 1
      },
      "counts": {
        "person_id": 1,
        "post_count": 0,
        "post_score": 0,
        "comment_count": 0,
        "comment_score": 0
      }
    }
  ]
}
```

---

## 📊 Account Information APIs

### 11. Get Account Information (includes wallet)
**GET** `/api/v4/account`

**Headers:**
```json
{
  "Authorization": "Bearer YOUR_JWT_TOKEN"
}
```

**Response:**
```json
{
  "local_user_view": {
    "local_user": {
      "id": 123,
      "person_id": 456,
      "walletId": 1,
      "email": "user@example.com",
      "show_nsfw": false,
      "theme": "browser",
      "default_sort_type": "Active",
      "default_listing_type": "Local",
      "interface_language": "browser",
      "show_avatars": true,
      "send_notifications_to_email": false,
      "validator_time": "2025-08-03T02:30:00Z",
      "show_bot_accounts": true,
      "show_scores": true,
      "show_read_posts": true,
      "email_verified": false,
      "accepted_application": false,
      "open_links_in_new_tab": false,
      "blur_nsfw": true,
      "auto_expand": true,
      "infinite_scroll_enabled": true,
      "admin": false,
      "post_listing_mode": "List",
      "totp_2fa_enabled": false,
      "enable_keyboard_navigation": false,
      "enable_animated_images": true,
      "collapse_bot_comments": false
    },
    "person": {
      "id": 456,
      "name": "your_username",
      "display_name": null,
      "avatar": null,
      "banned": false,
      "published": "2025-08-03T02:30:00Z",
      "updated": null,
      "actor_id": "http://localhost:8536/u/your_username",
      "bio": null,
      "local": true,
      "banner": null,
      "deleted": false,
      "inbox_url": "http://localhost:8536/u/your_username/inbox",
      "shared_inbox_url": "http://localhost:8536/inbox",
      "matrix_user_id": null,
      "admin": false,
      "bot_account": false,
      "ban_expires": null,
      "instance_id": 1
    },
    "counts": {
      "id": 456,
      "person_id": 456,
      "post_count": 0,
      "post_score": 0,
      "comment_count": 0,
      "comment_score": 0
    }
  },
  "community_blocks": [],
  "instance_blocks": [],
  "person_blocks": [],
  "keyword_blocks": [],
  "discussion_languages": [0],
  "wallet": {
    "id": 1,
    "balance": 1000.50,
    "escrowBalance": 250.00,
    "created_at": "2025-08-03T02:30:00Z",
    "updated_at": null
  }
}
```

---

## 🌐 Site Information API

### 11. Get Site Information (Public)
**GET** `/api/v4/site`

**Headers:**
```json
{
  "Content-Type": "application/json"
}
```

**Response:**
```json
{
  "site_view": {
    "site": {
      "id": 1,
      "name": "FastWork",
      "sidebar": "Welcome to FastWork freelancer marketplace",
      "published": "2025-08-03T02:00:00Z",
      "updated": null,
      "icon": null,
      "banner": null,
      "description": "Freelancer marketplace with escrow payment system",
      "actor_id": "http://localhost:8536/",
      "last_refreshed_at": "2025-08-03T02:00:00Z",
      "inbox_url": "http://localhost:8536/site_inbox",
      "private_key": null,
      "public_key": "-----BEGIN PUBLIC KEY-----...",
      "instance_id": 1
    },
    "local_site": {
      "id": 1,
      "site_id": 1,
      "site_setup": true,
      "enable_downvotes": true,
      "registration_mode": "Open",
      "enable_nsfw": true,
      "community_creation_admin_only": false,
      "require_email_verification": false,
      "application_question": null,
      "private_instance": false,
      "default_theme": "browser",
      "default_post_listing_type": "Local",
      "legal_information": null,
      "hide_modlog_mod_names": true,
      "application_email_admins": false,
      "slur_filter_regex": null,
      "actor_name_max_length": 20,
      "federation_enabled": true,
      "captcha_enabled": false,
      "captcha_difficulty": "medium",
      "published": "2025-08-03T02:00:00Z",
      "updated": null,
      "registration_closed": false,
      "enable_content_warning": false,
      "default_post_listing_mode": "List",
      "default_sort_type": "Active"
    },
    "counts": {
      "id": 1,
      "site_id": 1,
      "users": 3,
      "posts": 1,
      "comments": 0,
      "communities": 1,
      "users_active_day": 3,
      "users_active_week": 3,
      "users_active_month": 3,
      "users_active_half_year": 3
    }
  },
  "admins": [],
  "version": "1.0.0-alpha.5",
  "my_user": null,
  "all_languages": [],
  "discussion_languages": [0],
  "taglines": [],
  "custom_emojis": []
}
```

---

## 🚨 Error Responses

### Common Error Format
```json
{
  "error": "error_type",
  "message": "Human readable error message"
}
```

### Example Error Responses

**401 Unauthorized:**
```json
{
  "error": "not_logged_in",
  "message": "User not logged in"
}
```

**400 Bad Request:**
```json
{
  "error": "invalid_field",
  "message": "Price must be positive"
}
```

**403 Forbidden:**
```json
{
  "error": "not_an_admin", 
  "message": "User is not an admin"
}
```

**404 Not Found:**
```json
{
  "error": "not_found",
  "message": "Billing record not found"
}
```

---

## 📝 Field Validation Rules

### Create Invoice Fields:
- **price**: Must be positive number (> 0)
- **revise_times**: Must be non-negative integer (>= 0)
- **working_days**: Must be positive integer (> 0)
- **work_steps**: Array of strings, at least 1 item
- **deliverables**: Array of strings, at least 1 item
- **starting_day**: ISO date string (YYYY-MM-DD)
- **delivery_day**: ISO date string (YYYY-MM-DD)
- **proposal**: Required, non-empty string
- **name**: Required, non-empty string
- **job_description**: Required, non-empty string

### Wallet Operations:
- **amount**: Must be positive number (> 0)

---

## 🔄 Status Flow

### Billing Status Progression:
```
QuotationPending → OrderApproved → PaidEscrow → WorkSubmitted → Completed
                                              ↓
                                          RevisionRequested
                                              ↓
                                          WorkSubmitted (again)
```

### Alternative Endings:
- **Disputed**: If there are conflicts
- **Cancelled**: If quotation/order is cancelled

---

## 🧪 Quick Test Commands

### Test User Registration:
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123","password_verify":"test123","email":"test@example.com","show_nsfw":false}'
```

### Test Wallet Check:
```bash
curl -X GET http://localhost:8536/api/v4/account/wallet \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Test Quotation Creation:
```bash
curl -X POST http://localhost:8536/api/v4/account/wallet/create-invoice \
  -H "Authorization: Bearer FREELANCER_JWT" \
  -H "Content-Type: application/json" \
  -d '{"employer_id":2,"post_id":1,"price":100.00,"proposal":"Test proposal","name":"Test Job","job_description":"Test description","work_steps":["Step 1","Step 2"],"revise_times":1,"revise_description":"Test revisions","working_days":5,"deliverables":["Test deliverable"],"starting_day":"2025-08-05","delivery_day":"2025-08-10"}'
```

---

This API reference provides all the exact endpoints, payloads, and expected responses for testing your freelancer payment system! 🚀