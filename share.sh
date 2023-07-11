#!/bin/bash

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "jq command not found. Please install jq to run this script."
    exit 1
fi

# Check if GITHUB_API_TOKEN is set
if [ -z "$GITHUB_API_TOKEN" ]; then
    echo "GITHUB_API_TOKEN environment variable not found. Please set the token and try again."
    exit 1
fi

# Set the repository owner and repository name
OWNER="wokebuild"
REPO="share"
ASSET_NAME="wokeshare-v0.0.4.tar.gz"

# Set your GitHub personal access token
TOKEN="$GITHUB_API_TOKEN"

# Get the version argument
VERSION="$1"

if [ -z "$VERSION" ]; then
    echo "Please provide a version argument."
    exit 1
fi

# Retrieve the release information using the GitHub API
RELEASE_INFO=$(curl -s -H "Authorization: Bearer $TOKEN" "https://api.github.com/repos/$OWNER/$REPO/releases/tags/$VERSION")

# Extract the download URL from the release information
DOWNLOAD_URL=$(echo "$RELEASE_INFO" | jq -r ".assets[] | select(.name == \"$ASSET_NAME\") | .url")

if [ -z "$DOWNLOAD_URL" ]; then
  echo "Asset not found: $ASSET_NAME"
  exit 1
fi

# Download the asset using cURL with authentication headers
curl -L -H "Authorization: Bearer $TOKEN" -H "Accept: application/octet-stream" -o "$ASSET_NAME" -C - "$DOWNLOAD_URL"

# Check if the file was downloaded successfully
if [ ! -s "$ASSET_NAME" ]; then
  echo "Failed to download the asset: $ASSET_NAME"
  exit 1
fi

# Extract and rename the tar.gz file
tar -xzf "$ASSET_NAME" --transform 's/^udi-pgp-sqld/udi/'

# Clean up the downloaded tar.gz file
rm "$ASSET_NAME"
