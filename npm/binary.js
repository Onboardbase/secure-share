
const { Binary } = require("binary-install");
const os = require("os");
const cTable = require("console.table");

const error = msg => {
  console.error(msg);
  process.exit(1);
};

const { version } = require("./package.json");
const name = "scs";

const supportedPlatforms = [
  {
    TYPE: "Windows_NT",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-windows",
    BINARY_NAME: "scs.exe"
  },
  {
    TYPE: "Linux",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-linux",
    BINARY_NAME: "scs"
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-macos",
    BINARY_NAME: "scs"
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "arm64",
    RUST_TARGET: "x86_64-macos",
    BINARY_NAME: "scs"
  }
];

const getPlatformMetadata = () => {
  const type = os.type();
  const architecture = os.arch();

  for (let supportedPlatform of supportedPlatforms) {
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform;
    }
  }

  error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${cTable.getTable(
      supportedPlatforms
    )}`
  );
};

const getBinary = () => {
  const platformMetadata = getPlatformMetadata();
  const url = `https://github.com/onboardbase/secure-share/releases/download/v${version}/secure-share-v${ version }-${ platformMetadata.RUST_TARGET }.tar.gz`;
  return new Binary(platformMetadata.BINARY_NAME, url, version);
};

const run = () => {
  const binary = getBinary();
  binary.run();
};

const install = () => {
  const binary = getBinary();
  binary.install();
};

module.exports = {
  install,
  run
};