<div align="center">

# Secure share [![codecov](https://codecov.io/gh/Onboardbase/secure-share/branch/main/graph/badge.svg?token=H4CB88WT9I)](https://codecov.io/gh/Onboardbase/secure-share) [![Release](https://github.com/Onboardbase/secure-share/actions/workflows/release.yml/badge.svg)](https://github.com/Onboardbase/secure-share/actions/workflows/release.yml)[![Lint](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml/badge.svg)](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml)

Share anything with teammates across machines via CLI. Share is a tool for secure peer-to-peer connections, enabling direct communication and efficient exchange of secrets, files, and messages between machines with or without direct access to the internet. 

<!-- With Share, you can send something P2P, so you don't have to store whatever you want to share on somebody else's server. You don't have to set up a new server to transfer files to a teammate. -->
</div>

# Contents

- [Dependencies](#dependencies)
- [Install](#install)
- [Usage](#usage)
  - [Files](#files)
  - [Messages](#messages)
  - [Configuration](#configuration)
    - [Whitelists](#whitelists)
    - [Signed Certs](#SignedCertificate)
- [Update](#update)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Technical Details](#techincals)

## Dependencies

- `bash`, `curl`, `tar`: install these utilities.

## Install 
To use `scs`,
```bash
yarn add @onboardbase/secure-share # npm i @onboardbase/secure-share
```
Or, if you have rust on your machine:
```bash
cargo install scs
```
Or, using curl:
```sh
curl https://onboardbase.github.io/secure-share-sh/ | bash
```
Notes:
- For Windows users, please use `Git Bash` or any other CLI with the Bourne Shell.
- For users with Rust on their machines, ensure that `$HOME/.cargo/bin` directory is in your `$PATH` if you installed Rust with `rustup`. If not, please find the corresponding directory and add it to your `$PATH`.
and then,
```shell
scs --help
```
You should get a response displaying the utilities for `scs`
```
Share anything with teammates across machines via CLI.

Usage: scs [OPTIONS] <MODE>

Arguments:
  <MODE>  The mode (send secrets, or receive secrets). e,g `scs send` or `scs receive`
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
          Port to establish a connection on
  -d, --debug...
          Turn debugging information on
  -h, --help
          Print help
  -V, --version
          Print version
```


## Usage
`scs` enables the transmission of secrets or messages between teammates using different machines and behind different networks. To share a secret, the sender and receiver must get `scs` as described above and follow the instructions below.

#### The receiver:
Open a terminal or `cd` to where `scs` was installed, then:
```shell
scs receive
```
`scs` starts in listen mode and assigns you a `PeerId`, and picks a random port to start on. (An optional `-p` flag is available to specify a port). A response like the one below should be displayed:
```
INFO  Your PeerId is: 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt
INFO  Listening on "/ip4/172.19.192.1/tcp/54654"
INFO  Listening on "/ip4/192.168.0.197/tcp/54654"
INFO  Listening on "/ip4/127.0.0.1/tcp/54654"
INFO  Listening on "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt"
```

#### The sender:
Obtain the `PeerId` of the teammate you wish to send a secret to, then:
```shell
scs send -r 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt -s "hello, world"
```
`scs` will print your IP address and your `PeerId`.
To verify that a connection was established and your machine can talk to your teammates, you should see a similar thing below in your terminal:
```
INFO  Your PeerId is: 12D3KooWRpqX3QUvPNHXW5utkceLbx2b1LKfuAKa3iLdXXBGB2bY
INFO  Listening on "/ip4/127.0.0.1/tcp/40479"
INFO  Listening on "/ip4/192.168.212.254/tcp/40479"
INFO  Established connection to 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt via /ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt
```

The sender then attempts to send the secret, and if it is successful, `scs` relays  messages to both parties, notifying them of the status and the progress of the secret sharing session.

  ## Files
  `scs` also supports sending files:
  ```shell
  scs send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -f ../path/to/file1 -f path/to/file2
  ```
  ## Messages
  Ordinary messages can also be shared
  ```shell
  scs send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -m "hi there" -m "foo"
  ```
  All three items can also be sent together.

  ## Configuration
  As of `v0.0.12`, `scs` allows a configuration file to be passed. Ports, whitelists, and items can all be configured directly instead of passing them as arguments. A sample configuration file can be found [here](./config.yml). For example:

```yaml
port: 5555 #An optional port defaults to 0 if not present
# The folder path to store all items.
# Secrets will be stored at <path>/secrets.json
# Messages at <path>/messages.txt
# Files at <path>/nameoffile
## If "default" is passed, the folder path will be `scs`'s directory in the machine's local folder.
save_path: "default"
secret: #Optional during receive
  - key: foo
    value: bar
  - key: baz
    value: woo
message: #Optional during receive
  - new message from me
  - test message
file: #Optional during receive
  - "./dev_build.sh"
debug: 1 #Compulsory. 0 is for off, and 1 and above for on
blacklists:
  - 34.138.139.178
whitelists:
  - 34.193.14.12
```

  ```shell
  scs receive -c ./config.yml
  ```
  Or for senders:

  ```shell
  scs send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -c ./config.yml
  ```
 ### Whitelists/Blacklists IP addresses
 Whitelisting and blacklisting control traffic from specified IPs. To enable this feature, add the IP list to the config file. If no whitelist IPs are provided, all connections are allowed. However, if whitelist IPs are specified, only traffic from those addresses is permitted. Generic IPs like 127.0.0.1 (localhost) or 192.0.0.0 (firewall access points) won't work.
 ### Signed Certificate
 Receivers can configure `scs` to only allow connections from users that are using a signed cerificate from the CA. Or from just self sogned certificates. 
 To do this, add a `connection: trusted` or `connection: self` to the configuration file.

# Contributing

Contributions of any kind are welcome! See the [contributing guide](contributing.md).

[Thanks goes to these contributors](https://github.com/Onboardbase/secure-share/graphs/contributors)!

# Roadmap

### Utilities
- [ ] Personalize peer ID + allow saving recipient info (address, port, etc.) and giving a proper name so one can do "scs send dante -m Hello"
- [ ] Allow the possibility to always listen to specific addresses to have a free flow of data.


### Security
- [x] Signed Certificates from Let's Encrypt.

### Protocols
- [ ] Support QUIC. Use QUIC as default and fall back to TCP
- [ ] AutoNat: If you look closely, `scs` assumes both peers are behind NATs, firewalls, or proxies. But sometimes, this might not be the case, and it is excessive to hole punch just for that. Implementing `AutoNat` will first check if the two peers can communicate directly. If not, it will then proceed to hole punch. With TCP, this might take about 3 to 10 seconds, and this is where QUIC comes in and improves upon `scs`'s speed.

### Miscellaneous
- [ ] Send via disposable tunnel links + curl command to an API endpoint without local download (a way to "curl" on the consumer side so I can send them a link)

# License

See [LICENSE](LICENSE) © [Onboardbase](https://github.com/Onboardbase/)

# Technicals

The significant technical detail `scs` employs under the hood is P2P sharing. Below are excellent and detailed resources on P2P sharing and hole punching. Happy reading!!
  - https://blog.ipfs.tech/2022-01-20-libp2p-hole-punching/
  - https://tailscale.com/blog/how-nat-traversal-works/
