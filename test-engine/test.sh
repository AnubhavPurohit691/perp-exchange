#!/bin/bash

# Test script for http-backend
# This script runs the test-engine against the http-backend

echo "=== Test Engine Script ==="
echo ""
echo "This script will test the http-backend by:"
echo "  1. Creating users"
echo "  2. Creating orders (trades)"
echo "  3. Testing liquidation scenarios"
echo "  4. Testing funding rate effects"
echo ""
echo "Make sure http-backend is running on http://localhost:3000"
echo "You can start it with: cargo run --package http-backend"
echo ""
read -p "Press Enter to continue or Ctrl+C to cancel..."

echo ""
echo "Running test-engine..."
cargo run --package test-engine

