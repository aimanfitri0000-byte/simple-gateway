# mock_services.py - Sample microservices for testing
from fastapi import FastAPI
import uvicorn

# User Service (port 8001)
user_app = FastAPI()

@user_app.get("/users")
async def get_users():
    return {"users": ["Alice", "Bob", "Charlie"]}

@user_app.post("/users")
async def create_user():
    return {"message": "User created", "id": 123}

@user_app.get("/users/{user_id}")
async def get_user(user_id: int):
    return {"id": user_id, "name": f"User {user_id}", "email": f"user{user_id}@example.com"}

# Product Service (port 8002)
product_app = FastAPI()

@product_app.get("/products")
async def get_products():
    return {"products": ["Laptop", "Mouse", "Keyboard"]}

@product_app.get("/products/{product_id}")
async def get_product(product_id: int):
    return {"id": product_id, "name": f"Product {product_id}", "price": 100}

# Order Service (port 8003)
order_app = FastAPI()

@order_app.get("/orders")
async def get_orders():
    return {"orders": [{"id": 1, "total": 50}, {"id": 2, "total": 75}]}

@order_app.post("/orders")
async def create_order():
    return {"message": "Order created", "order_id": 456}

# Run all services
if __name__ == "__main__":
    import threading
    
    # Run user service
    threading.Thread(target=lambda: uvicorn.run(user_app, port=8001)).start()
    
    # Run product service  
    threading.Thread(target=lambda: uvicorn.run(product_app, port=8002)).start()
    
    # Run order service
    threading.Thread(target=lambda: uvicorn.run(order_app, port=8003)).start()
    
    print("Mock services running on ports 8001, 8002, 8003")
    print("Press Ctrl+C to stop")