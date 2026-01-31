# Context Test

This file tests working directory and environment variable persistence.

## 1. Change Directory
```bash
cd src
ls
```

## 2. Set Variable
```bash
export COMPASS_MODE=debug
```

## 3. Verify Persistence
```bash
pwd
echo $COMPASS_MODE
```
