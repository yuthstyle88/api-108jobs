# Authentication Debugging Guide

## Step 1: Check Site Status
First, verify the site is accessible:

```bash
curl -X GET http://localhost:8536/api/v4/site
```

**Expected**: Site information should be returned, not an error.

## Step 2: Check if Site Setup is Required
The site might need initial setup. Try this:

```bash
curl -X POST http://localhost:8536/api/v4/site \
  -H "Content-Type: application/json" \
  -d '{
    "name": "FastWork",
    "description": "Freelancer marketplace with escrow payment system",
    "icon": null,
    "banner": null,
    "enable_downvotes": true,
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
    "registration_closed": false,
    "enable_content_warning": false,
    "default_post_listing_mode": "List",
    "default_sort_type": "Active",
    "registration_mode": "Open"
  }'
```

## Step 3: Try Simple Registration
Test with minimal required fields:

```bash
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser1",
    "password": "password123",
    "password_verify": "password123"
  }'
```

## Step 4: If Registration Works, Try Login
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "testuser1",
    "password": "password123"
  }'
```

## Common Issues and Solutions:

### Issue 1: Site Not Setup
**Solution**: Run the site creation API call above.

### Issue 2: Registration Mode Closed
**Error**: Registration might be disabled.
**Solution**: Check site settings or use admin account.

### Issue 3: Password Requirements
**Error**: Password might not meet requirements.
**Solution**: Use longer password (8+ characters).

### Issue 4: Username Conflicts
**Error**: Username might already exist.
**Solution**: Use unique usernames.

### Issue 5: Email Verification Required
**Error**: Site might require email verification.
**Solution**: Disable email verification in site settings.

## Debug Commands:

### Check Database Connection:
```bash
# Check if server can connect to database
# Look in server logs for connection errors
```

### Check Server Logs:
Look for error messages in the server console output.

### Test with Different User:
```bash
curl -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "debuguser",
    "password": "debugpass123",
    "password_verify": "debugpass123",
    "email": "debug@test.com"
  }'
```

## If All Else Fails:
1. Restart the server
2. Check database is running
3. Check .env file configuration
4. Look at server console for detailed error messages