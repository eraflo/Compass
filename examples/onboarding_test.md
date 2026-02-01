# Interactive Setup Guide

This guide simulates a real-world onboarding scenario with variable inputs.

## 1. Welcome

We will configure your development environment.

```bash
echo "Welcome to the interactive setup!"
echo "Date: $(date)"
```

## 2. User Configuration

Please provide your details.

```bash
echo "Configuring user: <USERNAME>"
echo "Email: <EMAIL>"
```

## 3. Project Setup

We will create a project directory for `<PROJECT_NAME>`.

```bash
echo "Creating project directory..."
mkdir -p "./<PROJECT_NAME>"
cd "./<PROJECT_NAME>"
echo "Initialized project in $(pwd)"
```

## 4. Environment Check

Checking installed tools.

```bash
echo "Checking Node.js version..."
node --version || echo "Node.js not found"

echo "Checking Python version..."
python --version || echo "Python not found"
```
