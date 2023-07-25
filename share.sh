#!/bin/sh

# This script installs share.
#
# Quick install: `curl https://onboardbase.github.io/secure-share-sh/ | bash`
#
# This script will install share to the directory you're in. To install
# somewhere else (e.g. /usr/local/bin), cd there and make sure you can write to
# that directory, e.g. `cd /usr/local/bin; curl https://onboardbase.github.io/secure-share-sh/ | sudo bash`
#
# Found a bug? Report it here: https://github.com/onboardbase/secure-share/issues
#
# Acknowledgments:
#  - getmic.ro: https://github.com/benweissmann/getmic.ro
#  - eget: https://github.com/zyedidia/eget

set -e -u

githubLatestTag() {
  finalUrl=$(curl "https://github.com/$1/releases/latest" -s -L -I -o /dev/null -w '%{url_effective}')
  printf "%s\n" "${finalUrl##*v}"
}

platform=''
machine=$(uname -m)

# Check the GETSHARE_PALTFORM is set
if [ "${GETSHARE_PLATFORM:-x}" != "x" ]; then
  platform="$GETSHARE_PLATFORM"
else
  case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
    "linux")
      case "$machine" in
        "arm64"* | "aarch64"* ) platform='aarch64-linux' ;;
        "arm"* | "aarch"*) platform='aarch64-linux' ;;
        *"86") platform='x86_64-linux' ;;
        *"64") platform='x86_64-linux' ;;
      esac
      ;;
    "darwin")
      case "$machine" in
        "arm64"* | "aarch64"* ) platform='x86_64-macos' ;;
        *"64") platform='x86_64-macos' ;;
      esac
      ;;
    "msys"*|"cygwin"*|"mingw"*|*"_nt"*|"win"*)
      case "$machine" in
        *"86") platform='x86_64-windows' ;;
        *"64") platform='x86_64-windows' ;;
      esac
      ;;
  esac
fi

if [ "$platform" = "" ]; then
  cat << 'EOM'
/=====================================\\
|      COULD NOT DETECT PLATFORM      |
\\=====================================/
Uh oh! We couldn't automatically detect your operating system.
To continue with installation, please choose from one of the following values:
- x86_64-linux
- aarch64-linux
- x86_64-macos
- x86_64-windows
Export your selection as the GETSHARE_PLATFORM environment variable, and then
re-run this script.
For example:
  $ export GETSHARE_PLATFORM=linux_amd64
  $ curl https://woke.build/share | bash
EOM
  exit 1
else
  printf "Detected platform: %s\n" "$platform"
fi

TAG=$(githubLatestTag onboardbase/secure-share)

if [ "$platform" = "x86_64-windows" ]; then
  extension='zip'
else
  extension='tar.gz'
fi

printf "Latest Version: %s\n" "$TAG"
printf "Downloading https://github.com/onboardbase/secure-share/releases/download/v%s/secure-share-v%s-%s.%s\n" "$TAG" "$TAG" "$platform" "$extension"
curl -L "https://github.com/onboardbase/secure-share/releases/download/v$TAG/secure-share-v$TAG-$platform.$extension" > "share.$extension"

case "$extension" in
  "zip") unzip -j "share.$extension" -d "secure-share-v$TAG-$platform" ;;
  "tar.gz") tar -xvzf "share.$extension" "secure-share-v$TAG-$platform/share" ;;
esac

mv "secure-share-v$TAG-$platform/share" ./share

rm "share.$extension"
rm -rf "secure-share-v$TAG-$platform"

##Make share globally executable
executable="share"

# Check the operating system
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
  # Windows
  bin_dir="$HOME/bin"
  executable_path="$bin_dir/$executable"

  # Create the bin directory if it doesn't exist
  if [ ! -d "$bin_dir" ]; then
    mkdir "$bin_dir"
  fi

  # Copy the executable to the bin directory
  cp "$executable" "$executable_path"

  # Set executable permissions
  chmod +x "$executable_path"

  # Add the bin directory to the PATH environment variable
  echo "export PATH=\$PATH:$bin_dir" >> "$HOME/.bashrc"

elif [[ "$OSTYPE" == "linux-gnu"* || "$OSTYPE" == "darwin"* ]]; then
  # Linux or macOS
  executable_path="/usr/local/bin/$executable"

  # Copy the executable to the /usr/local/bin directory
  sudo cp "$executable" "$executable_path"

  # Set executable permissions
  sudo chmod +x "$executable_path"

else
  # Unsupported operating system
  echo "Unsupported operating system: $OSTYPE"
  exit 1
fi

rm -rf $executable

cat <<-'EOM'
Share has been downloaded and is now globally accessible.
You can run it with:
share --help
EOM