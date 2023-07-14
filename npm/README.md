<div align="center">

# share 

Share anything with teammates across machines via CLI
</div>

# Contents

- [Install](#install)
- [Usage](#usage)
- [Documentation](#documentation)
- [License](#license)



## Install 
```
yarn add @wokebuild/share # npm i @wokebuild/share
```

```bash
share --help
```
You should get a reponse displaying the utilties for `share`
```
Share anything with teammates across machines via CLI.

Usage: share [OPTIONS] <MODE>

Arguments:
  <MODE>  The mode (send secrets, or receive secrets). e,g `share send` or `share receive`
Options:
  -s, --secret <SECRET>
          Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
  -m, --message <MESSAGE>
          List of messages or a message string to deliver to the receiver. e,g -m "Hi there" -m "See me" or -m "hi there", "See me"
  -f, --file <FILE>
          List of file paths of files to deliver to the receiver. e,g -m "/path/to/file1" -m "../path/to/file2" or -m "path/to/file1", "../path/to/file2"
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
`share` enables transmission of secrets or messages between teammates using different machines and behind different networks. In order to share a secret, the sender and reciver need to both get `share` as described above and then follow the below instructions.

#### The receiver:
Open a terminal or `cd` to where `share` was installed, then:
```shell
./share receive
```
`share` starts in listen mode and assigns you a `PeerId` and picks a random port to start on. (An optional `-p` flag is available to specify a port). A reponse like below should be displayed:
```
  2023-07-14T07:40:44.706053Z  INFO share::hole_puncher: Your PeerId is: 12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR

  2023-07-14T07:40:44.712711Z  INFO share::hole_puncher: Listening on "/ip4/127.0.0.1/tcp/35459"

  2023-07-14T07:40:44.712996Z  INFO share::hole_puncher: Listening on "/ip4/172.19.198.56/tcp/35459"

  2023-07-14T07:40:49.934254Z  INFO share::hole_puncher: Listening on "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR"
```

#### The sender:
Obtain the `PeerId` of the teammate you wish to send a secret to, then:
```shell
./share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -s "hi,welcome"
```
`share` will print your IP address and your `PeerId`.
To verify that a connection was established and your machine can talk to your teammates, you should see a similar thing like below in your terminal:
```
  2023-07-14T07:43:09.253746Z  INFO share::hole_puncher: Your PeerId is: 12D3KooWSKcHXUAmp6FLkgJQ2DM1qVLDkeapXcHdUGtoHuLtqfgc

  2023-07-14T07:43:09.256025Z  INFO share::hole_puncher: Listening on "/ip4/127.0.0.1/tcp/44335"

  2023-07-14T07:43:09.256189Z  INFO share::hole_puncher: Listening on "/ip4/192.168.10.236/tcp/44335"

  2023-07-14T07:43:16.097707Z  INFO share::hole_puncher: Established connection to 12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR via /ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWBgJMyM6Akfx5hZcaa3F6zXVCpQykNXGqs96pDi4L71DR  
```
The sender then attempts to send the secret and if it is successful, `share` relays  messages to both parties notifying them of status and the progress of the secret sharing session.

# Documentation
Read more here https://github.com/wokebuild/share#readme

# License

See [LICENSE](LICENSE) © [Woke Build](https://github.com/wokebuild/)