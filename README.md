<div align="center">

# Woke share [![Release](https://github.com/wokebuild/share/actions/workflows/release.yml/badge.svg)](https://github.com/wokebuild/share/actions/workflows/release.yml)[![Lint](https://github.com/wokebuild/share/actions/workflows/lint.yml/badge.svg)](https://github.com/wokebuild/share/actions/workflows/lint.yml)

Share anything with teammates across machines via CLI
</div>

# Contents

- [Dependencies](#dependencies)
- [Install](#install)
- [Usage](#usage)
  - [Files](#files)
  - [Messages](#messages)
- [Update](#update)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Technical Details](#techincals)

## Dependencies

- `bash`, `curl`, `tar`: install these utilities.

## Install 
To use `share`,
```bash
yarn add @wokebuild/share # npm i @wokebuild/share
```
Or, using curl:
```sh
curl https://wokebuild.github.io | bash
```
Note:  This script will install `share` to your directory. To install it somewhere else (e.g.,/usr/local/bin), `cd` there and make sure you can write to that directory, e.g. 
```sh
cd /usr/local/bin
curl https://wokebuild.github.io | sudo bash
```
and then,
```shell
share --help # ./share --help if you used the bash script
```
You should get a response displaying the utilities for `share`
```
Share anything with teammates across machines via CLI.

Usage: share [OPTIONS] <MODE>

Arguments:
  <MODE>  The mode (send secrets, or receive secrets). e,g `share send` or `share receive`
Options:
  -s, --secret <SECRET>
          Separated list of secrets to share. The key-Value pair is separated by a comma. "my_key,my_value"
  -m, --message <MESSAGE>
          List of messages or a message string to deliver to the receiver. e,g -m "Hi there" -m "See me"
  -f, --file <FILE>
          List of file paths of files to deliver to the receiver. e,g -f "/path/to/file1" -f "../path/to/file2"
  -r, --remote-peer-id <REMOTE_PEER_ID>
          Peer ID of the remote to send secrets to
  -p, --port <PORT>
          Port to establish connection on
  -d, --debug...
          Turn debugging information on
  -h, --help
          Print help
  -V, --version
          Print version
```


## Usage
`share` enables the transmission of secrets or messages between teammates using different machines and behind different networks. To share a secret, the sender and receiver must both get `share` as described above and follow the instructions below.

#### The receiver:
Open a terminal or `cd` to where `share` was installed, then:
```shell
./share receive
```
`share` starts in listen mode and assigns you a `PeerId`, and picks a random port to start on. (An optional `-p` flag is available to specify a port). A response like the one below should be displayed:
```
  2023-07-14T07:40:44.706053Z  INFO share::hole_puncher: Your PeerId is: 12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR

  2023-07-14T07:40:44.712711Z  INFO share::hole_puncher: Listening on "/ip4/127.0.0.1/tcp/35459"

  2023-07-14T07:40:44.712996Z  INFO share::hole_puncher: Listening on "/ip4/172.19.198.56/tcp/35459"

  2023-07-14T07:40:49.934254Z  INFO share::hole_puncher: Listening on "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR"
```

#### The sender:
Obtain the `PeerId` of the teammate you wish to send a secret to, then:
```shell
./share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -s "hello, woke"
```
`share` will print your IP address and your `PeerId`.
To verify that a connection was established and your machine can talk to your teammates, you should see a similar thing below in your terminal:
```
  2023-07-14T07:43:09.253746Z  INFO share::hole_puncher: Your PeerId is: 12D3KooWSKcHXUAmp6FLkgJQ2DM1qVLDkeapXcHdUGtoHuLtqfgc

  2023-07-14T07:43:09.256025Z  INFO share::hole_puncher: Listening on "/ip4/127.0.0.1/tcp/44335"

  2023-07-14T07:43:09.256189Z  INFO share::hole_puncher: Listening on "/ip4/192.168.10.236/tcp/44335"

  2023-07-14T07:43:16.097707Z  INFO share::hole_puncher: Established connection to 12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR via /ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR  
```
The sender then attempts to send the secret, and if it is successful, `share` relays  messages to both parties, notifying them of the status and the progress of the secret sharing session.

  ## Files
  `share` also supports sending files:
  ```shell
  share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -f ../path/to/file1 -f path/to/file2
  ```
  ## Messages
  Ordinary messages can also be shared
  ```shell
  share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -m "hi there" -m "foo"
  ```
  All three items can also be sent together.

# Contributing

Contributions of any kind are welcome! See the [contributing guide](contributing.md).

[Thanks goes to these contributors](https://github.com/wokebuild/share/graphs/contributors)!

# Roadmap

### Utilities
- [ ] Configuration File: Enables users to pass in a config file as an argument instead of listing all parameters manually.
  - [ ] Default path to save items(messgaes, secrets and files).
  - [ ] Add a whitelist of IPs to allow connection from

### Security
- [ ] Signed Certificates from Let's Encrypt.
- [ ] Whitelists and Blacklists

### Protocols
- [ ] Support QUIC. Use QUIC as default and fall back to TCP
- [ ] AutoNat: If you look closely, `share` assumes both peers are behind NATs, firewalls, or proxies. But sometimes, this might not be the case, and it is excessive to hole punch just for that. Implementing `AutoNat` will first check if the two peers can communicate with each other directly. If not, it will then proceed to hole punch. With TCP, this might take about 3 to 10 seconds, and this is where QUIC comes in and improves upon `share`'s speed.

# License

See [LICENSE](LICENSE) © [Woke Build](https://github.com/wokebuild/)

# Technicals

The major technical detail `share` employs under the hood is P2P sharing. Below are great and detailed resources on P2P sharing and hole punching. Happy reading!!
  - https://blog.ipfs.tech/2022-01-20-libp2p-hole-punching/
