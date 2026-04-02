# auth.py - Simple authentication
from fastapi import HTTPException, Request
import jwt
from datetime import datetime, timedelta

# Secret key (in real app, use environment variable)
SECRET_KEY = "my-secret-key-change-this"
ALGORITHM = "HS256"

def create_token(user_id: str):
    """Create JWT token"""
    expire = datetime.utcnow() + timedelta(hours=24)
    payload = {
        "sub": user_id,
        "exp": expire,
        "iat": datetime.utcnow()
    }
    token = jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)
    return token

def verify_token(token: str):
    """Verify JWT token"""
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        return payload
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=401, detail="Token expired")
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid token")