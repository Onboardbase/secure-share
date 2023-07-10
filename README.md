<div align="center">

# share [![Release](https://github.com/wokebuild/share/actions/workflows/release.yml/badge.svg)](https://github.com/wokebuild/share/actions/workflows/release.yml)

Share anything with teammates across machines via CLI
</div>

# Contents

- [Dependencies](#dependencies)
- [Install](#install)
- [Usage](#usage)
- [Update](#update)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Dependencies

- `bash`, `curl`, `tar`: install these utilities.

## Install 
To use `share`, head over to the [releases](https://github.com/wokebuild/share/releases) to download the corresponding asset for your machine.

If you prefer the terminal, get your system architecture (`x86_64-linux` for linux, `x86_64-macos` or `x86_64-windows`) and run the below:
```shell
curl -L https://github.com/wokebuild/share/releases/download/v0.0.6/wokeshare-v0.0.6-<architecture>.tar.xz -o wokeshare.tar.xz
```
```shell
tar -xf wokeshare.tar.xz
cd wokeshare_v0.0.6-x86_64_linux/
/.share --help
```
You should get a reponse displaying the utilties for `share`
```
Share secrets securely through the terminal

Usage: share [OPTIONS] <MODE>

Arguments:
  <MODE>  The mode (share secrets, or rceive secrets)

Options:
  -s, --secret <SECRET>
          Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
      --remote-peer-id <REMOTE_PEER_ID>
          Peer ID of the remote to send secrets to
  -p, --port <PORT>
          Port to establish connection on
  -h, --help
          Print help
  -V, --version
          Print version
```


## Usage
`share` enables transmission of secrets or messages between teammates using different machines and behind different networks. In order to share a secret, the sender and reciver need to both get `share` as described above and then follow the below instructions.

- The receiver
Open a terminal or `cd` to where `share` was installed, then:
```shell
./share receive
```
`share` starts in listen mode and assigns you a `PeerId` and picks a random port to start on. (An optional `-p` flag is available to specify a port). A reponse like below should be displayed:
```
  2023-07-10T23:05:09.740636Z  INFO share::hole_puncher: Local peer id: PeerId("12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9")
    at src/hole_puncher.rs:43

  2023-07-10T23:05:09.746010Z  INFO share::hole_puncher: Listening on "/ip4/127.0.0.1/tcp/43467"
    at src/hole_puncher.rs:146

  2023-07-10T23:05:09.746230Z  INFO share::hole_puncher: Listening on "/ip4/172.19.198.56/tcp/43467"
    at src/hole_puncher.rs:146

  2023-07-10T23:05:12.150534Z  INFO share::hole_puncher: Told relay its public address.
    at src/hole_puncher.rs:173

  2023-07-10T23:05:12.656417Z  INFO share::hole_puncher: Relay told us our public address: "/ip4/102.89.41.179/tcp/36093"
    at src/hole_puncher.rs:180

  2023-07-10T23:05:13.645101Z  INFO share::hole_puncher: Relay accepted our reservation request.
    at src/hole_puncher.rs:221

  2023-07-10T23:05:13.645304Z  INFO share::hole_puncher: Listening on "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9"
    at src/hole_puncher.rs:215
```

- The sender
Obtain the `PeerId` of the teammate you wish to send a secret to, then:
```shell
./share send --remote-peer-id 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -s "hi,welcome"
```
`share` will print your IP address and your `PeerId`.
To verify that a connection was established and your machine can talk to your teammates, you should see a similar thing like below in your terminal:
```
  2023-07-10T23:10:47.574752Z  INFO share::hole_puncher: OutboundCircuitEstablished { relay_peer_id: PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"), limit: None }
    at src/hole_puncher.rs:223

  2023-07-10T23:10:49.753576Z  INFO share::hole_puncher: Established connection to PeerId("12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9") via Dialer { address: "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9", role_override: Dialer }
    at src/hole_puncher.rs:235
```
The sender then attempts to send the secret and if it is successful, `share` relays  messages to both parties notifying the of status and the progress of the secret sharing session.

# Contributing

Contributions of any kind welcome! See the [contributing guide](contributing.md).

[Thanks goes to these contributors](https://github.com/wokebuild/share/graphs/contributors)!

# License

See [LICENSE](LICENSE) © [Woke Build](https://github.com/wokebuild/)