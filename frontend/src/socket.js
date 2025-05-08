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
    let code =  ENV.state ? "StatefulQuery" : "StatelessQuery";
    let message = new Message(code, clock, "");

    clock += 1n;
    const msg = await sendAndWait(message.serialize())

    const cmd = msg.get_command();
    if (cmd === "Error") {
        throw new Error("Received error from backend");
    } else if (cmd !== "Acknowledge") {
        throw new Error("Received something strange from backend");
    }

    const array = JSON.parse(msg.data);
    
    for (const ser_change of array) {
        let buffer =  Buffer.from(ser_change) 
        change_fn(buffer)
    }

    if(ENV.debug){
        console.log("Queried Changes("+ array.length+ ") applied succesfuly")
    }
}

export async function save(data) {
    let message = new Message("Update", clock, data);
    clock += 1n;
    const ret = await sendAndWait(message.serialize())
    const cmd = ret.get_command();

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
}
