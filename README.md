# hermes

# Installation Guide
## Install requirements
```bash
cargo install --path .
```

## Build
```bash
cargo build --release
```

## Change directories
```bash
cd target
cd release
```

## Create .env file
```bash
# .env
# your api url
API_URL=https://your-url.com/api

# debug mode logs additional messages during runtime
DEBUG=true
```

## Run
```bash
hermes.exe
```