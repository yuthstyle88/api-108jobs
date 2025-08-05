#!/usr/bin/env python3
import jwt
import time
import uuid
import psycopg2
import json

# JWT Secret from database
JWT_SECRET = "ec4e55f8-ee1b-4964-a167-f6dbdf6bc505"

def create_jwt_token(user_id, role="User", email=None, lang="en"):
    """Create a JWT token for testing"""
    now = int(time.time())
    session = str(uuid.uuid4()).replace('-', '')
    
    payload = {
        "sub": str(user_id),
        "iss": "fastjob_dev",
        "iat": now,
        "exp": now + (12 * 3600),  # 12 hours
        "session": session,
        "role": role,
        "email": email,
        "lang": lang
    }
    
    token = jwt.encode(payload, JWT_SECRET, algorithm='HS256')
    
    # We also need to insert this token into the login_token table
    try:
        conn = psycopg2.connect(
            host="localhost",
            database="fastwork-new", 
            user="postgres",
            password="postgres"
        )
        cur = conn.cursor()
        
        # Insert into login_token table
        cur.execute(
            "INSERT INTO login_token (token, user_id, ip, user_agent) VALUES (%s, %s, %s, %s)",
            (token, user_id, "127.0.0.1", "curl/8.7.1")
        )
        conn.commit()
        cur.close()
        conn.close()
        
        return token
    except Exception as e:
        print(f"Error inserting token: {e}")
        return token

if __name__ == "__main__":
    # Create tokens for existing users
    employer_token = create_jwt_token(8, "User")  # employer_demo 
    freelancer_token = create_jwt_token(9, "User")  # freelancer_demo
    admin_token = create_jwt_token(10, "Admin")  # admin_demo
    
    print("Generated JWT tokens:")
    print(f"Employer (ID 8): {employer_token}")
    print(f"Freelancer (ID 9): {freelancer_token}")
    print(f"Admin (ID 10): {admin_token}")