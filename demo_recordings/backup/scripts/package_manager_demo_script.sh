#!/bin/bash
# Demo script for SentientOS Universal Package Manager feature
clear
echo -e "\033[1;36m$ cd /home/umesh/Sentinent_os\033[0m"
sleep 1
echo -e "\033[1;36m$ bin/sentctl package list\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installed packages:"
echo "┌───────────────┬────────────┬──────────────┬────────────────────┐"
echo "│ ECOSYSTEM     │ PACKAGE    │ VERSION      │ LOCATION           │"
echo "├───────────────┼────────────┼──────────────┼────────────────────┤"
echo "│ npm           │ express    │ 4.18.2       │ projects/api-gate~ │"
echo "│ npm           │ socket.io  │ 4.7.2        │ projects/api-gate~ │"
echo "│ python        │ flask      │ 2.3.3        │ projects/data-pro~ │"
echo "│ python        │ pandas     │ 2.1.0        │ projects/data-pro~ │"
echo "│ rust          │ tokio      │ 1.28.1       │ projects/sync-ser~ │"
echo "│ go            │ gin        │ 1.9.1        │ projects/auth-ser~ │"
echo "│ java          │ spring-boot│ 3.1.2        │ projects/payment-~ │"
echo "└───────────────┴────────────┴──────────────┴────────────────────┘"
sleep 3

echo -e "\033[1;36m$ bin/sentctl package install python --path projects/analytics\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing packages for Python project..."
sleep 1
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Detected requirements.txt"
sleep 1
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing numpy==1.24.3"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing matplotlib==3.7.2"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing scikit-learn==1.3.0"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing pandas==2.1.0"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Configuring package interoperability..."
sleep 1
echo -e "\033[1;32m✓ All packages installed successfully\033[0m"
sleep 2

echo -e "\033[1;36m$ bin/sentctl package verify projects/analytics\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Package Verifier]\033[0m Verifying package integrity..."
sleep 1
echo -e "\033[1;34m[SentientOS Package Verifier]\033[0m Checking for known vulnerabilities..."
sleep 1.5
echo -e "\033[1;32m✓ No vulnerabilities found\033[0m"
echo -e "\033[1;32m✓ All packages verified\033[0m"
sleep 2
