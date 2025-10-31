#!/bin/bash

# Script to run migrations and seed the Neon database
# Usage: ./scripts/seed_db.sh

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL is not set"
    echo "Please set it in your .env file or environment"
    exit 1
fi

echo "🔄 Running database migrations..."
psql "$DATABASE_URL" -f migrations.sql

echo ""
echo "🌱 Seeding vocabulary data..."
psql "$DATABASE_URL" -f seed_vocabulary.sql

echo ""
echo "✅ Database setup completed successfully!"
