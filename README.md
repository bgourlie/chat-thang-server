## EC2 Setup

This guide assumes your host is running the the Amazon Linux AMI.

#### Create security group for inbound traffic

TODO

#### Setup iptables to route port 80/443 into non-root bindable ports

** This step is only required if you want to accept connections on port 80 or 443 **

TODO

#### Install prerequisites

- `$ sudo yum install git`
- `$ sudo yum install gcc`

#### Install and configure the Rust toolchain

Install [rustup](https://www.rustup.rs/). Be sure to `source` the cargo environment script afterward:

    $ source $HOME/.cargo/env
    
You should also add the above line to your `.bashrc`.

#### Download and run

Clone the chat-thang-sources into a directory of your choosing:

    $ git clone https://github.com/bgourlie/chat-thang-server.git
    $ cd chat-thang-server
    
We will be using `cargo run` to compile and run the server:

    $ RUST_LOG=info cargo run -- --ip <host_private_ip> --port 8080
    
The above line does three things:

- `RUST_LOG=info`: Set the log level.  Valid values are `debug`, `error`, `info`, `warn`, or `trace`.
- `cargo run`: This command invokes cargo (rust's package manager) and will download dependencies, compile and run the server.
- ` -- --ip <host_private_ip> --port 8080`: Pass arguments to the server executable. These flags will configure the server to listen on the host's public IP address on port 8080. `<host_private_ip>` is a placeholder, you will need to bind to your host's *private* IP address in order to accept connections on the public IP address.
