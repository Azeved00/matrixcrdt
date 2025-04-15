# Protocol for comunication between frontend and backend

The comunication between the frontend (crdt, in this case javascript) and backend (Auth Dag)
is done by sockets.

## Base

The base of the protocol is length-prefixed messages.
Making use of a clock, for message ordering, and
an operation id, for differentiating between the different operations (more about operations later)

| 1 byte | 8 bytes | 8 bytes | n bytes|
|---|---|---| --- |
| operation code | message clock | message length | message |

### Notes
1. all integers(clock, length and operation specific)
are coded in **big endian** format.

2. `|message| == n`

## Operations

| Operation id |  Operation  |
| ------------ | ----------- |
| 0            | Acknowledge |
| 1            | Error       |
| 2            | Update      |
| 3            | Statefull Query       | 
| 4            | Stateless Query       | 

Any code other than the ones in the table will be treated as `Unknown`
All operations are initiated by the frontend to which the answer will come synchronosly

### Update Operation 

The `Update` operation is emmited by the frontend and it must include a binary message which will be added to the Auth Dag.

The backend will then return either an `Acknowledge`, in the case the update is successful, and 
an `Error` otherwise.

### Query Operation

There are 2 different `Query` operations both are emmited by the frontend and do not expect any data.

The difference between them is that a `Stateless Query`, queries every node of the DAG, while  
a `Statefull Query` queries only new nodes.


When a query is successful, its `Acknowledge` will contain an Array of binary messages representing the changes that were querried.
This array will be in the form:

| 8 bytes | 8 bytes | $n_0$ bytes | ...| 8 bytes | $n_i$ bytes|
|---|---|---| --- | --- | --- |
| number of changes | message $n_0$ size | $n_0$ message| ... |message $n_0$ size | $n_i$ message|

note that `number of changes == i`

In case of failure, the `Error` operation will be returned

### Acknowledge

Acknowledges are emited by the backend in case of a successful operation. 


### Error

Errors are emited only by the backend but in the case of an unsuccessful operation. 
As message errors include, in utf-8, why the operation failed.

## Clock

The message clock is used for ordering the messages as well as identifying the answer for each request.
When an `Update` or `Query` is requested, the answer will contain the same clock.

