
const { Binary } = require('binary-install');
const os = require('os');

function getPlatform() {
    const type = os.type();
    const arch = os.arch();

    if (type === 'Windows_NT' && arch === 'x64') return 'x86_64-windows';
    if (type === 'Linux' && arch === 'x64') return 'x86_64-linux';
    if (type === 'Darwin' && arch === 'x64') return 'x86_64-macos';

    throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function getBinary() {
    const platform = getPlatform();
    const version = require('../package.json').version;
    const url = `https://github.com/wokenuild/share/releases/download/wokeshare-v${ version }-${ platform }.tar.gz`;
    const name = 'share';
    return new Binary(url, { name });
}

module.exports = getBinary;