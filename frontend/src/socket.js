import net      from "net";
import * as ENV from './env.js';

const socket = new net.Socket();
let isConnected = false;
export let clock = 0n;

import Message from "./message.js";

export function init(port, addr) {
    socket.connect(port, addr, () => {
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
}

// Function to send data and wait for response
function sendAndWait(message) {
    return new Promise((resolve, reject) => {
        if (!isConnected) return reject("Socket not connected");

        const onData = (data) => {
            socket.off('error', onError);
            if (ENV.debug) {
                console.log("Message received");
            }
            const msg =  Message.deserialize(data);
            if (ENV.debug) {
                console.log(msg);
            }
            resolve(msg);
        };

        const onError = (err) => {
            socket.off('data', onData);
            reject(err);
        };

        socket.once('data', onData);
        socket.once('error', onError);

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
