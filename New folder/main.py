# main.py - API Gateway Simple
from fastapi import FastAPI, Request, Response, HTTPException
import httpx
import uvicorn

# Create FastAPI app
app = FastAPI(title="My First API Gateway")

# Configure services (microservices yang kita nak route)
SERVICES = {
    "users": "http://localhost:8001",     # User service
    "products": "http://localhost:8002",  # Product service
    "orders": "http://localhost:8003",    # Order service
}

# Create HTTP client
client = httpx.AsyncClient()

@app.on_event("shutdown")
async def shutdown_event():
    await client.aclose()

@app.get("/")
async def root():
    """Welcome page"""
    return {
        "message": "Welcome to API Gateway!",
        "available_services": list(SERVICES.keys()),
        "endpoints": {
            "/users": "User service",
            "/products": "Product service", 
            "/orders": "Order service"
        }
    }

@app.api_route("/{service}/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"])
async def gateway(service: str, path: str, request: Request):
    """
    Main gateway function - routes requests to appropriate service
    """

    if service not in SERVICES:
        raise HTTPException(status_code=404, detail={"error": f"Service '{service}' not found"})

    service_url = SERVICES[service].rstrip("/")
    target_url = f"{service_url}/{path}"

    body = await request.body()

    outgoing_headers = {
        k: v
        for k, v in request.headers.items()
        if k.lower() not in {"host", "content-length", "transfer-encoding", "connection"}
    }

    try:
        response = await client.request(
            method=request.method,
            url=target_url,
            headers=outgoing_headers,
            content=body,
            params=request.query_params
        )

        response_headers = {
            k: v
            for k, v in response.headers.items()
            if k.lower() not in {"content-length", "transfer-encoding", "connection"}
        }

        return Response(
            content=response.content,
            status_code=response.status_code,
            headers=response_headers,
            media_type=response.headers.get("content-type")
        )

    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail={"error": f"Service '{service}' is not available"})
    except Exception as e:
        raise HTTPException(status_code=500, detail={"error": str(e)})

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "gateway": "running"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=3000)
    # Add this to main.py (after imports)
from auth import create_token, verify_token

# Add protected endpoints
@app.api_route("/protected/{service}/{path:path}", methods=["GET", "POST"])
async def protected_gateway(service: str, path: str, request: Request):
    """Protected endpoint with authentication"""
    
    # Check Authorization header
    auth_header = request.headers.get("Authorization")
    if not auth_header:
        return {"error": "Missing authorization header"}, 401
    
    # Extract token
    try:
        token = auth_header.split(" ")[1]
        payload = verify_token(token)
    except Exception as e:
        return {"error": str(e)}, 401
    
    # Forward request (same as before)
    if service not in SERVICES:
        return {"error": f"Service '{service}' not found"}, 404
    
    service_url = SERVICES[service]
    target_url = f"{service_url}/{path}"
    
    body = await request.body()
    
    try:
        response = await client.request(
            method=request.method,
            url=target_url,
            headers=dict(request.headers),
            content=body,
            params=request.query_params
        )
        
        return {
            "status_code": response.status_code,
            "data": response.json() if response.headers.get("content-type") == "application/json" else response.text,
            "user": payload.get("sub")
        }
    except Exception as e:
        return {"error": str(e)}, 500

# Add endpoint to get token (for testing)
@app.post("/login")
async def login():
    """Get a test token"""
    token = create_token("test_user")
    return {"token": token, "type": "Bearer"}