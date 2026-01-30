-- Initialize TurboCommerce database schema
-- Run with: spin up --sqlite @migrations/init.sql

-- Products table
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    price_cents INTEGER NOT NULL,
    image_url TEXT,
    category TEXT,
    stock INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Cart items table (for persistent carts)
CREATE TABLE IF NOT EXISTS cart_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id)
);

-- Create index for cart lookups
CREATE INDEX IF NOT EXISTS idx_cart_session ON cart_items(session_id);

-- Seed initial product data
INSERT OR REPLACE INTO products (id, name, description, price_cents, image_url, category, stock) VALUES
    ('rust-book', 'Rust Programming Book', 'The complete guide to Rust programming language. Learn memory safety, ownership, and zero-cost abstractions.', 4999, '/images/rust_programming_book.png', 'books', 100),
    ('wasm-kit', 'WASM Development Kit', 'Everything you need to build WebAssembly applications. Includes tooling, examples, and best practices.', 9999, '/images/wasm-dev-kit.png', 'tools', 50),
    ('edge-guide', 'Edge Computing Guide', 'Master edge computing patterns. Deploy to Cloudflare Workers, Fermyon Cloud, and more.', 3999, '/images/edge_computing.png', 'books', 75),
    ('perf-pro', 'Performance Tuning Pro', 'Advanced performance optimization techniques for Rust applications. Profiling, benchmarking, and optimization.', 7999, '/images/spin-framework.png', 'tools', 30),
    ('turbo-course', 'TurboCommerce Mastery', 'Complete video course on building e-commerce with TurboCommerce. 40+ hours of content.', 19999, '/images/cargo_crate_stickers.png', 'courses', 999),
    ('leptos-book', 'Leptos in Action', 'Build reactive web applications with Leptos. Full-stack Rust for the modern web.', 5499, '/images/ferris_plushie.png', 'books', 60);
