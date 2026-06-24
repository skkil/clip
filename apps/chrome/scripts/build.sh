#!/usr/bin/env bash

set -e
set -u
set -o pipefail

npm run clean

npm run format
npm run lint

npm run compile

npm run build:popup
npm run build:background
npm run build:content

