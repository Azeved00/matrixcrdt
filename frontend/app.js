import net      from "net";
import express  from 'express';
import path     from "path";
import dcrdtLib from './lib/delta-crdt/frontend/index.js';
import MessageProcessor from "./src/processor.js";
import Message from "./src/message.js";
import msgpack from "@msgpack/msgpack";
import { BSON, EJSON, ObjectId } from 'bson';


const __dirname = path.resolve(path.dirname(''))
let dcrdt = dcrdtLib.init({});
const socket = new net.Socket();
const app = express();
const port = 3000;
let clock = 0n;
let msgProc = new MessageProcessor();
let last_change = "";

// Set up socket to Auth Dag
socket.connect(20076, "127.0.0.1", () => {
    console.log("Connected to Rust server!");
});
socket.on("data", (data) => {
    console.log("message received");
    let message = Message.deserialize(data);
    msgProc.processMessage(message);
});
socket.on("close", () => {
    console.log("Connection closed");
});

// Set up express server
app.use(express.json());

app.get('/', (_req, res) => {
  res.sendFile(__dirname + '/pages/index.html');
});

// Get all key-value pairs
app.get('/map', (_req, res) => {
    const docv = JSON.stringify(dcrdtLib.documentValue(dcrdt), (_key, value) => 
        value instanceof Set ? [...value] : value
    );
    console.log(docv);
    res.json(docv);
});

// Get value by key
app.get('/map/:key', (req, res) => {
    const key = req.params.key;
    if (dataMap.has(key)) {
        res.json({ key, value: dataMap.get(key) });
    } else {
        res.status(404).json({ error: 'Key not found' });
    }
});

// Add or update key-value pair
app.post('/map', (req, res) => {
    const { key, value } = req.body;
    if (!key || value === undefined) {
        return res.status(400).json({ error: 'Key and value are required' });
    }
    dcrdt = dcrdtLib.change(dcrdt, {}, (doc) => {
        doc.key = value;
    });
    res.json({ message: 'Entry added/updated', key, value });
});

// Delete a key-value pair
app.delete('/map/:key', (req, res) => {
    const key = req.params.key;
    dcrdt = dcrdtLib.change(dcrdt, {}, (doc) => {
      delete doc.key;
    });
    res.json({ message: 'Entry deleted', key });
});

app.get('/save', (_req, res) => {
    const delta = dcrdtLib.getChanges(dcrdt);
    last_change = delta;
    const ser_delta = msgpack.encode(delta);
    let message = new Message(0, clock, ser_delta);
    msgProc.enqueueCounter(clock, (_data) => {
        console.log("Saved Successfuly");
    });
    clock += 1n;
    socket.write(message.serialize())

    res.status(200).json();
});

app.get('/query', (_req, res) => {
    let message = new Message(1, clock, "");
    msgProc.enqueueCounter(clock, (data) => {
        const jsonString = data.toString("utf-8");
        const array = JSON.parse(jsonString);

        for (const ser_change of array) {
            let change = msgpack.decode(ser_change)
            console.log(last_change)
            console.log(change)
            dcrdtLib.applyChanges(dcrdt, change)
        }

        console.log("Queried Changes("+ array.length+ ") applied succesfuly")
    });
    clock += 1n;
    socket.write(message.serialize())

    res.status(200).json();
});



// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});



