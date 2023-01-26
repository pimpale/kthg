#!/bin/bash

./target/debug/kthg \
  --port=8080 \
  --database-url=postgres://postgres:toor@localhost/kthg \
  --auth-service-url=http://localhost:8079 \
  --app-pub-origin=http://localhost:3000
