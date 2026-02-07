#!/bin/bash

# Test script to verify the command parsing regression is fixed
echo "Testing command parsing fix..."

# Build the project
echo "Building project..."
cargo build --quiet

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Build successful!"

# Test the commands by running the binary directly with a timeout
echo "Testing commands..."

# Create a temporary input file
cat > temp_input.txt << 'EOF'
testcommander
systems
info LEESTI  
jump LEESTI
quit
EOF

# Run the game with timeout to prevent hanging
timeout 10s ./target/debug/elitey_rust < temp_input.txt > temp_output.txt 2>&1

# Check if the output contains expected results and no error messages
if grep -q "Error: System 'jump' not found" temp_output.txt; then
    echo "❌ REGRESSION DETECTED: jump command still broken"
    echo "Output:"
    cat temp_output.txt
    rm -f temp_input.txt temp_output.txt
    exit 1
elif grep -q "Error: System 'info' not found" temp_output.txt; then
    echo "❌ REGRESSION DETECTED: info command still broken"
    echo "Output:"
    cat temp_output.txt
    rm -f temp_input.txt temp_output.txt
    exit 1
elif grep -q "Available systems:" temp_output.txt; then
    echo "✅ SUCCESS: Commands are working correctly!"
    echo "Sample output:"
    head -20 temp_output.txt
    rm -f temp_input.txt temp_output.txt
    exit 0
else
    echo "⚠️  Unexpected output - please check manually:"
    cat temp_output.txt
    rm -f temp_input.txt temp_output.txt
    exit 1
fi
