# Protocol for comunication between frontend and backend

The comunication between the frontend (crdt, in this case javascript) and backend (Auth Dag)
is done by sockets.

## Base

The base of the protocol is length-prefixed messages.
Making use of a clock, for message ordering, and
an operation id, for differentiating between the different operations (more about operations later)

| 1-bit | 4-bit| 4-bit| n-bits|
|---|---|---| --- |
| operation code | message clock | message length | message |

note that `|message| == n`


## Operations

| Operation id |  Operation  |
| ------------ | ----------- |
| 0            | Update      |
| 1            | Query       | 
| 2            | Acknowledge |
| 3            | Error       |

Any code other than the ones in the table will be treated as `Unknown`
All operations are initiated by the frontend to which the answer will come synchronosly

### Update Operation 

The `Update` operation is emmited by the frontend and it must include a binary message which will be added to the Auth Dag.

The backend will then return either an `Acknowledge`, in the case the update is successful, and 
an `Error` otherwise.

### Query Operation

The `Query` operation is, also, emmited by the frontend but it doesnt require a message. 
Instead, in the case the query is successful, the `Acknowledge` will contain an Array of binary messages representing the changes that were querried.
This array will be in the form:

| 4-bit | 4-bit| $n_0$-bit| ...| 4-bit| $n_i$-bit|
|---|---|---| --- | --- | --- |
| number of changes | message $n_0$ size | $n_0$ message| ... |message $n_0$ size | $n_i$ message|

note that `number of changes == i`

In case of failure, the `Error` operation will be returned

### Acknowledge

Acknowledges are emited by the backend in case of a successful operation. 


### Error

Errors are emited only by the backend but in the case of an unsuccessful operation. 
As message errors include, in utf-8, why the operation failed.
