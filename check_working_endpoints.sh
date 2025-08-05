#!/bin/bash

# Script to check which wallet endpoints are currently working
BASE_URL="http://localhost:8536"

echo "=== Checking Available Wallet Endpoints ==="

# Test if /services endpoints are available
echo "Testing /services endpoints..."
SERVICES_TEST=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/v4/account/services/test")
if [ "$SERVICES_TEST" = "404" ]; then
    echo "❌ /services endpoints NOT available (404)"
    echo "✅ Use /wallet endpoints instead"
    CURRENT_ENDPOINTS="/wallet"
else
    echo "✅ /services endpoints available"
    CURRENT_ENDPOINTS="/services"
fi

echo ""
echo "=== Current Working Endpoints ==="
echo "Base path: /api/v4/account${CURRENT_ENDPOINTS}"
echo ""
echo "Available operations:"
echo "• POST ${CURRENT_ENDPOINTS}/create-invoice"
echo "• POST ${CURRENT_ENDPOINTS}/approve-quotation" 
echo "• POST ${CURRENT_ENDPOINTS}/submit-work"
echo "• POST ${CURRENT_ENDPOINTS}/request-revision"
echo "• POST ${CURRENT_ENDPOINTS}/approve-work"
echo ""

if [ "$CURRENT_ENDPOINTS" = "/wallet" ]; then
    echo "⚠️  Server restart needed to use /services endpoints"
    echo "💡 For now, use /wallet endpoints in your requests"
fi