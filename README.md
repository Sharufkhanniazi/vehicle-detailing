# 🚗 Vehicle Detailing Microservices Platform

A comprehensive **Rust-based microservices backend** for an on-demand **vehicle detailing service** (Uber-style service booking platform). The system supports user authentication, booking management, pricing estimates, real-time tracking, event-driven communication, and multi-channel notifications.

---

## 📋 Table of Contents

- [Architecture Overview](#-architecture-overview)
- [Services](#-services)
  - [Auth Service](#1️⃣-auth-service)
  - [Booking Service](#2️⃣-booking-service)
  - [Pricing Service](#3️⃣-pricing-service)
  - [Tracking Service](#4️⃣-tracking-service)
  - [Notification Service](#5️⃣-notification-service)
  - [Email Service](#6️⃣-email-service)
  - [Assign Detailer Service](#7️⃣-assign-detailer-service)
  - [Shared Auth Library](#8️⃣-shared-auth-library)
- [Event-Driven Communication](#-event-driven-communication)
- [Database Schema](#-database-schema)
- [Authentication](#-authentication)
- [Environment Variables](#-environment-variables)
- [Docker Setup](#-docker-setup)
- [Running the Project](#-running-the-project)
- [API Documentation](#-api-documentation)
- [Real-time Features](#-real-time-features)
- [Scalability](#-scalability)
- [Tech Stack](#-tech-stack)
- [Project Structure](#-project-structure)
- [Future Improvements](#-future-improvements)

---

## 🏗 Architecture Overview

This project implements a **microservices architecture** where each service handles a specific business capability, communicating through **asynchronous events** and **synchronous gRPC/HTTP** calls.

```
                                    +-------------------+
                                    |    API Gateway    |
                                    |  (Planned/Manual  |
                                    +---------+---------+
                                              |
        -------------------------------------------------------------------------
        |                 |                 |                 |                 |
+---------------+  +---------------+  +---------------+  +---------------+  +---------------+
|     Auth      |  |    Booking    |  |    Pricing    |  |   Tracking    |  | Notification  |
|   Service     |  |   Service     |  |   Service     |  |   Service     |  |   Service     |
|   (Port 3000) |  |   (Port 3001) |  |   (Port 50051)|  |   (Port 3002) |  |   (Port 3003) |
+---------------+  +---------------+  +---------------+  +---------------+  +---------------+
        |                  |                 |                 |                   |
        -------------------|-----------------|-----------------|--------------------
                           |                 |                 |
                    +------v------+    +------v------+   +-----v------+
                    | Assign Detailer   |    Email      |   |  Tracking   |
                    | Service           |    Service    |   |  Consumer   |
                    +-------------------+    (Consumer) |   +------------+
                           |                 +------------+
                           |                              |
                    +------v------------------------------v------+
                    |                 Apache Kafka                |
                    +------------------------------------------------+
                                                |
                    +---------------------------+--------------------------+
                    |                           |                          |
            +-------v-------+           +-------v-------+          +-------v-------+
            |  PostgreSQL   |           |     Redis     |          |  WebSocket    |
            |  (Persistence) |           |   (Caching)   |          |  Connections  |
            +---------------+           +---------------+          +---------------+
```

### Key Concepts

- **Event-Driven Architecture**: Services communicate asynchronously via Kafka
- **CQRS Pattern**: Separate read/write models where appropriate
- **Saga Pattern**: Distributed transactions across services
- **Database per Service**: Each service has its own database schema
- **Real-time Communication**: WebSockets for live tracking updates
- **Caching Layer**: Redis for frequently accessed location data
- **gRPC**: Efficient internal service communication

---

## 📦 Services

### 1️⃣ Auth Service

**Port: 3000**  
Handles user registration, authentication, and profile management.

#### Features

- User registration with email verification
- JWT-based authentication
- Role-based access control (CUSTOMER/DETAILER)
- Email verification workflow
- Password hashing with Argon2
- Kafka event publishing for user creation

#### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/register` | Register new user |
| POST | `/login` | User login |
| POST | `/verify-email` | Verify email with token |
| POST | `/resend/email` | Resend verification email |

#### Events Produced

- `user.created` - When a new user registers

---

### 2️⃣ Booking Service

**Port: 3001**  
Manages booking creation, cancellation, and order lifecycle.

#### Features

- Create service bookings
- Cancel bookings (customer/detailer)
- Mark orders as completed
- Integrate with pricing service via gRPC
- Kafka event publishing for booking events
- Kafka consumer for assignment events

#### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/price` | Get price estimate |
| POST | `/booking` | Create new booking |
| POST | `/cancel` | Cancel booking |
| POST | `/completed` | Mark order as completed |

#### Events Produced

- `booking.created` - When a new booking is made
- `booking.cancelled` - When a booking is cancelled

#### Events Consumed

- `detailer.assigned` - Detailer assigned to order
- `detailer.notfound` - No detailer available
- `detailer.arrived` - Detailer arrived at location

---

### 3️⃣ Pricing Service

**Port: 50051 (gRPC)**  
Calculates service price estimates with vehicle-based multipliers.

#### Features

- gRPC API for internal service communication
- Vehicle category pricing multipliers
- Multiple service type support
- Tax calculation (10%)
- Surge pricing logic
- Time-limited estimates (5 minutes)

#### Service Types

| Service | Base Price |
|---------|------------|
| Exterior Wash | 1500 PKR |
| Interior Clean | 2000 PKR |
| Full Detailing | 5000 PKR |
| Engine Bay Cleaning | 1800 PKR |

#### Vehicle Multipliers

| Vehicle Type | Multiplier |
|--------------|------------|
| Small | 1.0x |
| Sedan | 1.2x |
| SUV | 1.5x |
| Truck | 1.8x |

#### gRPC Methods

```protobuf
rpc GetEstimate (PriceEstimateRequest) returns (PriceEstimateResponse)
```

---

### 4️⃣ Tracking Service

**Port: 3002**  
Handles real-time location tracking between customers and detailers.

#### Features

- Real-time location updates via WebSockets
- Redis caching for location data
- Distance calculation (Haversine formula)
- ETA estimation
- Kafka consumer for booking cancellation events
- Broadcast location updates to connected clients

#### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/update-location` | Detailer updates location |
| GET | `/tracking/{order_id}` | Get tracking info |
| GET | `/tracking/ws/{order_id}` | WebSocket for live tracking |
| GET | `/tracking/distance` | Calculate distance between points |
| POST | `/tracking/arrival/{order_id}` | Notify detailer arrival |

#### WebSocket Messages

```json
{
  "type": "LocationUpdate",
  "order_id": "uuid",
  "latitude": 24.8607,
  "longitude": 67.0011,
  "distance_km": 5.2,
  "eta_minutes": 10,
  "timestamp": "2025-01-01T10:00:00Z"
}
```

#### Events Consumed

- `booking.cancelled` - Clean up tracking connections

#### Events Produced

- `detailer.arrived` - Detailer arrived notification

---

### 5️⃣ Notification Service

**Port: 3003**  
Manages multi-channel notifications (WebSocket, Push, Database).

#### Features

- WebSocket connections for real-time notifications
- Firebase Cloud Messaging (FCM) for push notifications
- Notification persistence in PostgreSQL
- Kafka consumer for various events
- Broadcast notifications to connected clients

#### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/ws/notifications` | WebSocket for notifications |

#### Events Consumed

- `booking.cancelled` - Booking cancelled notifications
- `detailer.assigned` - Detailer assigned notifications
- `detailer.notfound` - No detailer available
- `detailer.arrived` - Detailer arrived notifications

#### Notification Channels

1. **WebSocket** - Real-time for connected clients
2. **Push Notification** - FCM for mobile devices
3. **Database** - Persistent notification history

---

### 6️⃣ Email Service

**Consumer-only service**  
Handles email communications using Resend API.

#### Features

- Email verification emails
- Resend API integration
- Kafka consumer for user creation events

#### Events Consumed

- `user.created` - Send verification email

---

### 7️⃣ Assign Detailer Service

**Consumer-only service**  
Matches available detailers to booking requests.

#### Features

- Proximity-based detailer matching
- Haversine formula for distance calculation
- Availability status checking
- Double-booking prevention
- Rating and experience prioritization

#### Matching Algorithm

1. Filter detailers within bounding box (10km radius)
2. Check availability status (ONLINE)
3. Verify no conflicting bookings
4. Calculate exact distance
5. Sort by: distance ↑, rating ↓, completed jobs ↓
6. Assign best match

#### Events Consumed

- `booking.created` - Find available detailer

#### Events Produced

- `detailer.assigned` - Detailer found and assigned
- `detailer.notfound` - No available detailer

---

### 8️⃣ Shared Auth Library

**Internal crate**  
Reusable authentication and models shared across services.

#### Features

- JWT token validation
- Shared model definitions
- Kafka event schemas
- User role enumeration

#### Shared Models

```rust
UserRole { CUSTOMER, DETAILER }
Claims { sub, email, username, role, exp }
UserCreatedEvent
BookingCreatedEvent
BookingCancelledEvent
AssignedDetailerEvent
DetailerNotFoundEvent
DetailerArrivedEvent
```

---

## 📡 Event-Driven Communication

Services communicate asynchronously through **Apache Kafka** topics.

### Topics Overview

| Topic | Producer | Consumers | Description |
|-------|----------|-----------|-------------|
| `user.created` | Auth Service | Email Service | New user registered |
| `booking.created` | Booking Service | Assign Detailer Service | New booking created |
| `booking.cancelled` | Booking Service | Tracking, Notification | Booking cancelled |
| `detailer.assigned` | Assign Detailer Service | Booking, Notification | Detailer assigned |
| `detailer.notfound` | Assign Detailer Service | Booking, Notification | No detailer available |
| `detailer.arrived` | Tracking Service | Booking, Notification | Detailer arrived |

### Event Flow Examples

#### Booking Creation Flow

```
1. Customer creates booking → Booking Service
2. Booking Service → `booking.created` event
3. Assign Detailer Service ← `booking.created`
4. Assign Detailer Service → `detailer.assigned` or `detailer.notfound`
5. Booking Service ← `detailer.assigned` (updates order)
6. Notification Service ← `detailer.assigned` (notifies customer)
```

#### Real-time Tracking Flow

```
1. Detailer updates location → Tracking Service (HTTP)
2. Tracking Service stores in Redis & PostgreSQL
3. Tracking Service broadcasts via WebSocket to connected customers
4. Detailer arrives → Tracking Service → `detailer.arrived` event
5. Booking Service ← `detailer.arrived` (updates order status)
6. Notification Service ← `detailer.arrived` (notifies customer)
```

---

## 🗄 Database Schema

### PostgreSQL Tables

#### Users & Profiles
```sql
users (id, username, email, password_hash, role, is_email_verified, is_active, fcm_token)
customer_profiles (user_id, address, loyalty_points)
detailer_profiles (user_id, phone_number, rating, total_jobs_completed, availability_status, last_known_latitude, last_known_longitude)
```

#### Orders
```sql
orders (id, customer_id, detailer_id, brand, model, vehicle, time_slot, status, subtotal, tax, surge_multiplier, total_price, latitude, longitude)
order_services (id, order_id, service, base_price, final_price)
```

#### Notifications
```sql
notifications (id, user_id, title, body)
```

### Redis Keys

| Key Pattern | Purpose | TTL |
|-------------|---------|-----|
| `location:{detailer_id}` | Cached location | 300s |

---

## 🔐 Authentication

All protected endpoints require a **JWT Bearer Token**.

### JWT Structure

```json
{
  "sub": "uuid",        // User ID
  "email": "user@example.com",
  "username": "john_doe",
  "role": "CUSTOMER",   // or "DETAILER"
  "exp": 1742313600     // Expiration timestamp
}
```

### Authorization Header

```
Authorization: Bearer <jwt_token>
```

### Role-Based Access

- **CUSTOMER**: Can create bookings, track orders
- **DETAILER**: Can update location, complete orders

---

## ⚙️ Environment Variables

Create a `.env` file in each service directory with the following variables:

### Common Variables (All Services)
```env
DATABASE_URL=postgres://user:password@postgres:5432/vehicle_detailing
KAFKA_BROKER=kafka:9092
JWT_SECRET_KEY=your_super_secret_jwt_key_here
```

### Auth Service
```env
JWT_EXPIRATION=24  # hours
```

### Email Service
```env
RESEND_API_KEY=re_xxxxxxx
```

### Notification Service
```env
FCM_PROJECT_ID=your-firebase-project-id
```

### Tracking Service
```env
REDIS_URL=redis://redis:6379
```

### Assign Detailer Service
```env
# Uses common variables only
```

---

## 🐳 Docker Setup

The project includes comprehensive Docker configuration for all services.

### Docker Compose Structure

```yaml
services:
  # Databases
  postgres:
  redis:
  
  # Message Broker
  zookeeper:
  kafka:
  kafka-ui:
  
  # Microservices
  auth-service:
  booking-service:
  pricing-service:
  tracking-service:
  notification-service:
  email-service:
  assign-detailer-service:
```

### Building and Running

```bash
# Build all services
docker compose build

# Start all services
docker compose up -d

# View logs
docker compose logs -f [service-name]

# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v
```

### Service Ports

| Service | Port | Protocol |
|---------|------|----------|
| Auth Service | 3000 | HTTP |
| Booking Service | 3001 | HTTP |
| Tracking Service | 3002 | HTTP |
| Notification Service | 3003 | HTTP |
| Pricing Service | 50051 | gRPC |
| PostgreSQL | 5432 | TCP |
| Redis | 6379 | TCP |
| Kafka | 9092 | TCP |
| Kafka UI | 8080 | HTTP |

### Container Dependencies

```
auth-service: postgres, kafka
booking-service: postgres, kafka, pricing-service
pricing-service: (standalone)
tracking-service: postgres, redis, kafka
notification-service: postgres, kafka
email-service: kafka
assign-detailer-service: postgres, kafka
```

---

## 🚀 Running the Project

### Prerequisites

- Rust 1.70+
- Docker and Docker Compose
- PostgreSQL 14+
- Redis 7+
- Apache Kafka 3.5+

### Option 1: Docker (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd vehicle-detailing-platform

# Copy environment files
cp .env.example .env

# Start all services
docker compose up -d

# Check service status
docker compose ps
```

### Option 2: Local Development

```bash
# Start infrastructure
docker compose up -d postgres redis zookeeper kafka

# Run pricing service (gRPC)
cd pricing-service
cargo run

# In separate terminals, run each service
cd auth-service && cargo run
cd booking-service && cargo run
cd tracking-service && cargo run
cd notification-service && cargo run
cd email-service && cargo run
cd assign-detailer-service && cargo run
```

### Verify Installation

```bash
# Check Kafka topics
docker compose exec kafka kafka-topics --list --bootstrap-server localhost:9092

# Check PostgreSQL
docker compose exec postgres psql -U user -d vehicle_detailing -c "\dt"

# Test auth service
curl -X POST http://localhost:3000/health
```

---

## 📊 API Documentation

### Auth Service Examples

#### Register Customer
![alt text](register.jpeg)
```bash
curl -X POST http://localhost:3000/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "password123",
    "user_role": "CUSTOMER"
  }'
```

#### Login
![alt text](login.jpeg)
```bash
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "password": "password123"
  }'
```

### Booking Service Examples

#### Get Price Estimate
```bash
curl -X GET http://localhost:3001/price \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "vehicle": "Sedan",
    "services": ["ExteriorWash", "InteriorClean"]
  }'
```

#### Create Booking
![alt text](<booking.png>)
```bash
curl -X POST http://localhost:3001/booking \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "brand": "Toyota",
    "model": "Camry",
    "vehicle": "Sedan",
    "services": ["ExteriorWash"],
    "time_slot": "2025-01-01T10:00:00Z",
    "latitude": 24.8607,
    "longitude": 67.0011
  }'
```

### Tracking Service Examples

#### Update Location (Detailer)
![alt text](<location.png>)
```bash
curl -X POST http://localhost:3002/update-location \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "uuid",
    "latitude": 24.8607,
    "longitude": 67.0011,
    "timestamp": "2025-01-01T10:00:00Z"
  }'
```

#### WebSocket Connection (Customer)
```javascript
const ws = new WebSocket('ws://localhost:3002/tracking/ws/{order_id}');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Location update:', data);
};
```

---

## 🔄 Real-time Features

### WebSocket Connections

1. **Tracking WebSocket** (`/tracking/ws/{order_id}`)
   - Customers receive real-time location updates
   - Automatic reconnection handling
   - Ping/pong keepalive

2. **Notification WebSocket** (`/ws/notifications`)
   - Receive notifications in real-time
   - Multi-client support per user
   - Automatic cleanup on disconnect

### Location Tracking Algorithm

**Haversine Formula** for distance calculation:

```
a = sin²(Δlat/2) + cos(lat1) * cos(lat2) * sin²(Δlon/2)
c = 2 * atan2(√a, √(1-a))
distance = R * c  (R = 6371km, Earth's radius)
```

**ETA Calculation**:
- Average speed assumed: 30 km/h
- ETA minutes = (distance / 30) * 60

### Caching Strategy

- **Redis**: Detailer locations cached for 5 minutes
- **Fallback**: PostgreSQL if Redis unavailable
- **Cache invalidation**: On location updates

---

## 📈 Scalability

### Horizontal Scaling

- **Stateless Services**: Auth, Booking, Notification (scale with load balancer)
- **Stateful Tracking**: Redis helps manage WebSocket connections
- **Kafka Partitioning**: Topics partitioned for parallel consumption

### Performance Optimizations

1. **Database Indexing**
   - Location-based queries
   - Status filtering
   - Time slot lookups

2. **Caching Layers**
   - Redis for location data
   - Connection pooling
   - Prepared statements

3. **Async Processing**
   - Non-blocking I/O
   - Background task processing
   - Event-driven architecture

### Load Handling

- **Kafka Consumer Groups**: Multiple instances of same service
- **Database Connection Pooling**: Configurable max connections
- **WebSocket Broadcasting**: Efficient fan-out

---

## 🛠 Tech Stack

### Core Technologies

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Language** | Rust 2024 | Performance, safety, concurrency |
| **Web Framework** | Axum 0.8 | HTTP API, WebSockets |
| **gRPC** | Tonic 0.11 | Internal service communication |
| **Database** | PostgreSQL 14 | Persistent storage |
| **Cache** | Redis 7 | Location caching |
| **Message Broker** | Kafka 3.5 | Event streaming |
| **ORM** | SQLx 0.8 | Async, compile-time queries |
| **Authentication** | JSON Web Tokens | Stateless auth |
| **Password Hashing** | Argon2 | Secure password storage |

### Key Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| tokio | 1.49 | Async runtime |
| rdkafka | 0.36 | Kafka client |
| sqlx | 0.8.6 | PostgreSQL client |
| redis | 1.0.3 | Redis client |
| serde | 1.0 | Serialization |
| uuid | 1.21 | UUID generation |
| chrono | 0.4.44 | Date/time handling |
| dashmap | 6.1 | Concurrent hashmap |

---

## 📁 Project Structure

```
vehicle-detailing-platform/
├── Cargo.toml                    # Workspace definition
├── Cargo.lock                    # Locked dependencies
├── docker-compose.yml             # Docker composition
├── .env.example                   # Environment template
├── migrations/                    # Root database migrations
│   └── 20260227164156_create_tables.sql
├── proto/                         # Protocol Buffers
│   └── pricing.proto
├── shared-auth/                   # Shared authentication library
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── lib.rs
│       ├── jwt.rs
│       └── models.rs
├── auth-service/                  # Authentication service
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 20260320004214_test_db.sql
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   └── auth.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── kafka_producer.rs
│   │   │   └── metrics.rs
│   │   ├── utils/
│   │   │   ├── mod.rs
│   │   │   ├── error.rs
│   │   │   └── jwt.rs
│   │   └── models.rs
│   └── tests/
│       ├── integration_tests.rs
│       └── common/
│           ├── mod.rs
│           └── test_db.rs
├── booking-service/               # Booking management
│   ├── Cargo.toml
│   ├── build.rs
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 20260320035210_test_db.sql
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── state.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   └── booking.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── booking.rs
│   │   │   ├── consumer.rs
│   │   │   ├── kafka_producer.rs
│   │   │   └── metrics.rs
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth_user.rs
│   │   │   └── rate_limiting.rs
│   │   ├── utils/
│   │   │   ├── mod.rs
│   │   │   ├── error.rs
│   │   │   └── models.rs
│   │   └── proto/
│   │       └── mod.rs
│   └── tests/
│       ├── integration_tests.rs
│       └── common/
│           ├── mod.rs
│           └── test_db.rs
├── pricing-service/               # Price calculation
│   ├── Cargo.toml
│   ├── build.rs
│   ├── Dockerfile
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   └── tests/
│       └── integration_tests.rs
├── tracking-service/              # Real-time tracking
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 20260330100554_test_db.sql
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── state.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   └── tracking.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── tracking.rs
│   │   │   ├── consumer.rs
│   │   │   ├── kafka_producer.rs
│   │   │   └── metrics.rs
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth_user.rs
│   │   │   └── rate_limiting.rs
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── errors.rs
│   │       └── models.rs
│   └── tests/
│       ├── integration_tests.rs
│       └── common/
│           ├── mod.rs
│           └── test_db.rs
├── notification-service/          # Multi-channel notifications
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── src/
│   │   ├── main.rs
│   │   ├── consumer.rs
│   │   ├── handler.rs
│   │   ├── services.rs
│   │   ├── state.rs
│   │   ├── errors.rs
│   │   ├── util.rs
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth_user.rs
│   │   │   └── rate_limiting.rs
│   │   └── vehicle_detailing.json
│   └── vehicle_detailing.json
├── email-service/                  # Email communications
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── main.rs
│       ├── consumer.rs
│       └── mailer.rs
└── assign-detailer-service/        # Detailer matching
    ├── Cargo.toml
    ├── Dockerfile
    ├── migrations/
    │   └── 20260319044050_test_db.sql
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs
    │   ├── assign_detailer.rs
    │   ├── consumer.rs
    │   ├── errors.rs
    │   └── kafka_producer.rs
    └── tests/
        ├── integration_tests.rs
        └── common/
            ├── mod.rs
            └── test_db.rs
---

## Testing

The project implements a multi-layered testing strategy across all microservices, including unit tests, integration tests, and test database migrations.

### Test Structure

Each service follows a consistent test organization:
service-name/
├── tests/
│ ├── integration_tests.rs # Integration test suite
│ └── common/ # Shared test utilities
│ ├── mod.rs # Common module exports
│ └── test_db.rs # Test database setup & teardown
└── migrations/
└── *_test_db.sql # Service-specific test migrations

### Test Coverage by Service

| Service | Integration Tests | Test DB Migrations | Common Utils |
|---------|:----------------:|:------------------:|:------------:|
| Auth Service | ✓ | ✓ (20260320004214_test_db.sql) | ✓ |
| Booking Service | ✓ | ✓ (20260320035210_test_db.sql) | ✓ |
| Pricing Service | ✓ | - | - |
| Tracking Service | ✓ | ✓ (20260330100554_test_db.sql) | ✓ |
| Assign Detailer Service | ✓ | ✓ (20260319044050_test_db.sql) | ✓ |
| Notification Service | - | - | - |
| Email Service | - | - | - |

### Running Tests

#### Run all tests across workspace:
```bash
cargo test --workspace

## Rate Limiting

The platform implements distributed rate limiting across most microservices to prevent abuse, ensure fair usage, and protect system resources. Rate limiting is handled at the middleware level using Redis for distributed state management

## Prometheus Metrics

The platform integrates Prometheus for comprehensive observability, monitoring, and alerting across all microservices. Metrics are collected at the middleware level and exposed via a dedicated metrics endpoint.

---

## 👨‍💻 Author

Developed as a production-grade **microservices backend** for a real-time vehicle detailing platform, demonstrating advanced Rust backend engineering concepts including:

- **Event-driven architecture**
- **Real-time communication**
- **Distributed systems patterns**
- **CQRS and event sourcing**
- **Database optimization**
- **Concurrent programming**
- **Type-safe database operations**

---

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tonic](https://github.com/hyperium/tonic) - gRPC framework
- [rdkafka](https://github.com/fede1024/rust-rdkafka) - Kafka client
- All contributors to the Rust ecosystem