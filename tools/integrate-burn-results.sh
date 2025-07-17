#!/bin/bash

# SentientOS Oracle Burn Results Integration Tool
# This script helps integrate Oracle burn test results into your system

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

RESULTS_FILE="../burn_results/complete-oracle-burn-results.json"

# Check if results file exists
if [ ! -f "$RESULTS_FILE" ]; then
    echo -e "${RED}Error: Oracle burn test results file not found.${NC}"
    echo "Please run the Oracle burn test first using ./complete-oracle-burn.sh"
    exit 1
fi

echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}  SentientOS Burn Results Integrator ${NC}"
echo -e "${BLUE}====================================${NC}"

# Parse results file
echo "Parsing test results..."
OVERALL_STATUS=$(jq -r '.status' "$RESULTS_FILE")
PASS_PERCENTAGE=$(jq -r '.pass_percentage' "$RESULTS_FILE")

if [ "$OVERALL_STATUS" == "success" ]; then
    echo -e "${GREEN}Oracle burn test PASSED with ${PASS_PERCENTAGE}% success rate${NC}"
else
    echo -e "${YELLOW}Oracle burn test partially succeeded with ${PASS_PERCENTAGE}% success rate${NC}"
    echo "Detailed failure information:"
    jq -r '.results[] | select(.status == "failed") | "\(.name): \(.error)"' "$RESULTS_FILE" || true
fi

# Integration options
echo -e "\n${BLUE}Integration Options:${NC}"
echo "1. Copy results to system directory"
echo "2. Export as standard JSON"
echo "3. View detailed metrics"
echo "4. Generate system compatibility report"
echo "5. Exit"

read -p "Select an option (1-5): " OPTION

case $OPTION in
    1)
        echo "Integrating results into system..."
        DEST_DIR="/tmp/sentient_results"
        mkdir -p "$DEST_DIR"
        cp "$RESULTS_FILE" "$DEST_DIR/burn_results.json"
        echo -e "${GREEN}Results copied to $DEST_DIR/burn_results.json${NC}"
        echo "You can now integrate this file with your system"
        ;;
    2)
        EXPORT_FILE="../burn_results/burn_results_export.json"
        jq '.' "$RESULTS_FILE" > "$EXPORT_FILE"
        echo -e "${GREEN}Results exported to $EXPORT_FILE${NC}"
        ;;
    3)
        echo -e "${BLUE}Performance Metrics:${NC}"
        jq -r '.performance_metrics' "$RESULTS_FILE"
        echo -e "\n${BLUE}Individual Test Metrics:${NC}"
        jq -r '.results | to_entries[] | "\(.key): \(.value.success_rate)% success rate, \(.value.duration)s duration"' "$RESULTS_FILE"
        ;;
    4)
        COMPAT_FILE="../burn_results/system_compatibility.txt"
        echo "SentientOS System Compatibility Report" > "$COMPAT_FILE"
        echo "Generated: $(date)" >> "$COMPAT_FILE"
        echo "---------------------------------" >> "$COMPAT_FILE"
        echo "Overall Status: $OVERALL_STATUS (${PASS_PERCENTAGE}%)" >> "$COMPAT_FILE"
        echo "" >> "$COMPAT_FILE"
        echo "System Requirements:" >> "$COMPAT_FILE"
        echo "- Memory: At least 128MB RAM" >> "$COMPAT_FILE"
        echo "- Storage: At least 250MB free space" >> "$COMPAT_FILE"
        echo "- CPU: Any modern x64 or ARM64 processor" >> "$COMPAT_FILE"
        echo "" >> "$COMPAT_FILE"
        echo "Integration Recommendations:" >> "$COMPAT_FILE"
        if [ "$PASS_PERCENTAGE" = "100.00" ]; then
            echo "✓ System is READY for production deployment" >> "$COMPAT_FILE"
        elif [ $(echo "$PASS_PERCENTAGE >= 90.0" | bc -l) -eq 1 ]; then
            echo "⚠ System is suitable for BETA deployment" >> "$COMPAT_FILE"
        else
            echo "✗ System requires additional testing before deployment" >> "$COMPAT_FILE"
        fi
        
        echo -e "${GREEN}Compatibility report generated at $COMPAT_FILE${NC}"
        ;;
    5)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid option selected.${NC}"
        exit 1
        ;;
esac

echo -e "\n${GREEN}Integration complete!${NC}"
echo "You can now use the Oracle burn test results in your system."
