# Socket Binary

An expample of the how to use the `Merkle Dag` library.


## How does it work
This example open a socket for comunicating such that any application can use it.
Upon receiving data the program will perform the correct operation (Save or Query).

## How to use
Any way to interact with the socket can be used, as long as the protocol,
see [socket comunication protocol](../../../../artifacts/socket_protocol.md), 
is correctly used.

### Provided Node Frontend
We have provided an example frontend in Node.js that makes use of DCRDT.

For this its needed both cargo and node to be installed, and simply doing:
```
cargo run --bin socket
```

and
```
npm start
```

In two different terminals will be enough to get the example started which
the user can interact with the program trough the node js server (webpage or direct curl).

In this example the user interacts with the CRDT, and can save and query.
Both of these operations will send a message to trough the socket adding a node to the merkle dag and
gathering the unknown nodes, respectively.
