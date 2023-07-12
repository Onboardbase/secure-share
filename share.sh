#!/bin/sh

# This script installs share.
#
# Quick install: `curl https://woke.build/share | bash`
#
# This script will install share to the directory you're in. To install
# somewhere else (e.g. /usr/local/bin), cd there and make sure you can write to
# that directory, e.g. `cd /usr/local/bin; curl https://woke.build/share | sudo bash`
#
# Found a bug? Report it here: https://github.com/wokebuild/share/issues
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

TAG=$(githubLatestTag wokebuild/share)

if [ "$platform" = "x86_64-windows" ]; then
  extension='zip'
else
  extension='tar.gz'
fi

printf "Latest Version: %s\n" "$TAG"
printf "Downloading https://github.com/wokebuild/share/releases/download/v%s/wokeshare-v%s-%s.%s\n" "$TAG" "$TAG" "$platform" "$extension"
curl -L "https://github.com/wokebuild/share/releases/download/v$TAG/wokeshare-v$TAG-$platform.$extension" > "share.$extension"

case "$extension" in
  "zip") unzip -j "share.$extension" -d "wokeshare-v$TAG-$platform" ;;
  "tar.gz") tar -xvzf "share.$extension" "wokeshare-v$TAG-$platform/share" ;;
esac

mv "wokeshare-v$TAG-$platform/share" ./share

rm "share.$extension"
rm -rf "wokeshare-v$TAG-$platform"

cat <<-'EOM'
Share has been downloaded to the current directory.
You can run it with:
./share
EOM