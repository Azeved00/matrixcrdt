import net      from "net";

const isDev = process.env.NODE_ENV === 'dev';
const stateless = process.env.STATE_ENV === 'stateless';

const socket = new net.Socket();
let isConnected = false;
let clock = 0n;

import Message from "./message.js";

export function init(port, addr) {
    socket.connect(port, addr, () => {
        isConnected = true;
        console.log("Connected to Rust server!");
    });

    socket.on("close", () => {
        isConnected = false;
        console.log("Connection closed");
    });
}

// Function to send data and wait for response
function sendAndWait(message) {
    return new Promise((resolve, reject) => {
        if (!isConnected) return reject("Socket not connected");

        const onData = (data) => {
            socket.off('error', onError);
            if (isDev) console.log("Message received");
            const msg =  Message.deserialize(data);
            resolve(msg.data);
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
    let  code =  stateless ? 4 : 3;
    let message = new Message(code, clock, "");

    clock += 1n;
    const data = await sendAndWait(message.serialize())
    const array = JSON.parse(data);
    
    if(isDev){
        console.log("Received Response")
    }
    for (const ser_change of array) {
        let buffer =  Buffer.from(ser_change) 
        change_fn(buffer)
    }

    if(isDev){
        console.log("Queried Changes("+ array.length+ ") applied succesfuly")
    }
}

export async function save(data) {
    let message = new Message(0, clock, data);
    clock += 1n;
    const _return = await sendAndWait(message.serialize())
    if(isDev){
        console.log("Saved Successfuly");
    }
}
