import net      from "net";
import * as ENV from './env.js';

const socket = new net.Socket();
let isConnected = false;
export let clock = 0n;

let currentResolve = null;
let currentReject = null;
let buffer = Buffer.alloc(0);

import Message from "./message.js";

// Core buffering logic to handle full messages
function onData(chunk) {
    buffer = Buffer.concat([buffer, chunk]);
    if(ENV.debug){
        console.log(`got message, buffer has ${buffer.length} bytes`)
    }

    while (buffer.length >= Message.HEADER_SIZE) {
        const msg = Message.deserialize_header(buffer);
        const totalLength = Message.HEADER_SIZE + Number(msg.length);

        if (buffer.length < totalLength) return; 

        msg.data = buffer.slice(Message.HEADER_SIZE, totalLength);
        buffer = buffer.slice(totalLength);

        if (ENV.debug) {
            console.log("Message fully received");
            console.log(msg);
        }

        if (currentResolve) {
            currentResolve(msg);
            currentResolve = null;
            currentReject = null;
        }
    }
}

export function init(port, addr, localPort) {
    const lp = 2000 + parseInt(localPort, 10);
    socket.connect({
        host: addr,
        port: port,
        localAddress: '127.0.0.1',
        localPort: lp
    }, () => {
        isConnected = true;
        if (ENV.debug){
            console.log("Connected to Rust server!");
        }
    });

    socket.on("close", () => {
        isConnected = false;
        if (ENV.debug){
            console.log("Connection closed");
        }
    });

    socket.on("data", onData);
    socket.on("error", (err) => {
        if (currentReject) {
            currentReject(err);
            currentReject = null;
            currentResolve = null;
        }
    });
}



// Function to send data and wait for response
function sendAndWait(message) {
    clock += 1n;
    return new Promise((resolve, reject) => {
        if (!isConnected) return reject("Socket not connected");

        if (currentResolve !== null) {
            return reject("Previous request still pending");
        }

        currentResolve = resolve;
        currentReject = reject;


        socket.write(message);
    });
}

export async function query(change_fn) {
    const timing = {};

    const code = ENV.state ? "StatefulQuery" : "StatelessQuery";

    const startSerialize = process.hrtime.bigint();
    let message = new Message(code, clock, "");
    if(ENV.debug){
        console.log("Querying", message);
    }
    const serialized = message.serialize();
    const endSerialize = process.hrtime.bigint();

    const msg = await sendAndWait(serialized);

    
    const startDeserialize = process.hrtime.bigint();
    const cmd = msg.get_command();
    if (cmd === "Error") throw new Error("Received error from backend");
    else if (cmd !== "Acknowledge") throw new Error("Received something strange from backend");

    const array = JSON.parse(msg.data);
    const endDeserialize = process.hrtime.bigint();
    timing.serialization = Number(endSerialize - startSerialize) / 1000 +
        Number(endDeserialize - startDeserialize) / 1000;

    const start_apply = process.hrtime.bigint();
    for (const ser_change of array) {
        const buffer = Buffer.from(ser_change);
        change_fn(buffer);
    }
    const end_apply = process.hrtime.bigint();
    timing.state_apply  = Number(end_apply - start_apply) / 1000;

    if (ENV.debug) {
        console.log(`Queried Changes (${array.length}) applied successfully`);
        console.log("Timing Info:", timing);
    }

    return timing;
}

export async function save(data) {
    const timing = {};

    const startSerialize = process.hrtime.bigint();
    let message = new Message("Update", clock, data);
    if(ENV.debug){
        console.log("Sending new Update", message);
    }
    const endSerialize = process.hrtime.bigint();

    const ret = await sendAndWait(message.serialize())

    const startDeserialize = process.hrtime.bigint();
    const cmd = ret.get_command();
    const endDeserialize = process.hrtime.bigint();

    if(ENV.debug){
        console.log(cmd);
        console.log(ret.code);
    }

    if (cmd === "Error") {
        throw new Error("Received error from backend");
    } else if (cmd !== "Acknowledge") {
        throw new Error("Received something strange from backend");
    }

    if(ENV.debug){
        console.log("Updated Successfuly");
    }

    timing.serialization = Number(endSerialize - startSerialize) / 1000 +
        Number(endDeserialize - startDeserialize) / 1000;

    return timing;
}
