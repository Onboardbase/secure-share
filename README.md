<div align="center">

# Secure share [![codecov](https://codecov.io/gh/Onboardbase/secure-share/branch/main/graph/badge.svg?token=H4CB88WT9I)](https://codecov.io/gh/Onboardbase/secure-share) [![Lint](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml/badge.svg)](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml)

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
    - [Whitelists](#whitelistsblacklists-ip-addresses)
    - [Signed Certs](#signed-certificate)
    - [Seed Key](#seeds-seed-key)
- [Recipient Info](#saving-peer-info)
- [Storage](#items-storage-location)
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
save_path: "default"
secret: # Optional during receive
  - key: foo
    value: bar
  - key: baz
    value: woo
message: # Optional during receive
  - new message from me
  - test message
file: # Optional during receive
  - "./dev_build.sh"
debug: 1 # Compulsory. 0 is for off, and 1 and above for on
blacklists:
  - 34.138.139.178
whitelists:
  - 34.193.14.12
connection: trusted # or self
seed: "scsiscool"
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
 Receivers can configure `scs` to only allow connections from users using a signed certificate from the CA. or just self-signed certificates. 
 Add a `connection: trusted` or `connection: self` to the configuration file.

 ### Seeds (Seed Key)
 The backbone of `scs` is `PeerId`. A `PeerId` is a randomly generated key whenever a session is started for both the receiver and the sender. As of `v0.1.3` of `scs`, `PeerId`s can now be deterministic; a single `PeerId` can be used for life. To do this, you need to set a "seed". The `PeerId` is generated concerning this seed. As long as the seed key remains the same, the `PeerId` will remain. 
 The "seed" key is a string of any length lesser than 32. But for ease and optimal configuration, we recommend 4 or 5 letter words as in the above configuration file.


# Saving Peer Info
To make using `scs` easier after the initial setup, `scs` implements a simple mechanism for storing recipients' information. 
After every session with a new peer, `scs` asks if you'll like to save the information of the connected peer. If you decide to send to that same peer, pass in the name of the peer to the `-n` argument like below
```sh
scs send -n dante -c config.yml
```
Note: For security reasons, we don't save the IP addresses of the connected peers on each machine.

To see all saved peers:
```sh
scs list
```
# Items Storage Location
Items sent (secrets, files, and messages) are stored in the local folder on the machine. To find the saved items:
- Windows: `/c/Users/<name_of_user>/AppData/Local/onboardbase/secureshare/data`
- Linux: `/home/<name_of_user>/.local/share/secureshare`
- Mac: 
# Contributing

Contributions of any kind are welcome! See the [contributing guide](contributing.md).

[Thanks goes to these contributors](https://github.com/Onboardbase/secure-share/graphs/contributors)!

# Roadmap

### Utilities
- [ ] Allow to always listen to specific addresses for an accessible data flow.

### Protocols
- [ ] AutoNat: If you look closely, `scs` assumes both peers are behind NATs, firewalls, or proxies. But sometimes, this might not be the case, and it is excessive to hole punch just for that. Implementing `AutoNat` will first check if the two peers can communicate directly. If not, it will then proceed to hole punch. With TCP, this might take about 3 to 10 seconds, and this is where QUIC comes in and improves upon `scs`'s speed.

# License

See [LICENSE](LICENSE) © [Onboardbase](https://github.com/Onboardbase/)

# Technicals

The significant technical detail `scs` employs under the hood is P2P sharing. Below are excellent and detailed resources on P2P sharing and hole punching. Happy reading!!
  - https://blog.ipfs.tech/2022-01-20-libp2p-hole-punching/
  - https://tailscale.com/blog/how-nat-traversal-works/
