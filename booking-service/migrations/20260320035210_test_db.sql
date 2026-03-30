CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('CUSTOMER', 'DETAILER');

CREATE TYPE availability_status AS ENUM ('ONLINE', 'OFFLINE', 'BUSY');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email CITEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL,
    is_email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    fcm_token TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


CREATE TABLE customer_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    address VARCHAR(255),
    loyalty_points INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE detailer_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    phone_number VARCHAR(20),
    rating NUMERIC(3, 2) DEFAULT 0.00 CHECK (rating >= 0 AND rating <= 5),
    total_rating_points INT NOT NULL DEFAULT 0, 
    total_reviews INT NOT NULL DEFAULT 0,
    total_jobs_completed INT DEFAULT 0,
    availability_status availability_status DEFAULT 'ONLINE',
    last_known_latitude DOUBLE PRECISION NOT NULL DEFAULT 0.00,
    last_known_longitude DOUBLE PRECISION NOT NULL DEFAULT 0.00,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TYPE vehicle_category AS ENUM ('SMALL', 'SEDAN', 'SUV', 'TRUCK');

CREATE TYPE order_status AS ENUM (
    'PENDING',
    'AWAITING_PAYMENT',
    'CONFIRMED',
    'ASSIGNED',
    'IN_PROGRESS',
    'COMPLETED',
    'CANCELLED'
);

CREATE TYPE service_type AS ENUM (
    'ExteriorWash',
    'InteriorClean',
    'FullDetailing',
    'EngineBayCleaning'
);

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL REFERENCES users(id),
    detailer_id UUID REFERENCES users(id), -- Because when order is created: status = PENDING no detailer assigned yet
    brand TEXT NOT NULL,
    model TEXT NOT NULL,
    vehicle vehicle_category NOT NULL,
    time_slot TIMESTAMP WITH TIME ZONE NOT NULL,
    status order_status NOT NULL DEFAULT 'PENDING',
    subtotal NUMERIC(10,2) NOT NULL,
    tax NUMERIC(10,2) NOT NULL,
    surge_multiplier NUMERIC(5,2) NOT NULL DEFAULT 1.0,
    total_price NUMERIC(10,2) NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);


CREATE TABLE order_services (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,

    service service_type NOT NULL,

    base_price NUMERIC(10,2) NOT NULL,
    final_price NUMERIC(10,2) NOT NULL,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
