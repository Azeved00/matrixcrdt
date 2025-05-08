import net      from "net";
import express  from 'express';
import path     from "path";
import * as Automerge from "@automerge/automerge";
import Message from "./src/message.js";
import msgpack from "@msgpack/msgpack";

const __dirname = path.resolve(path.dirname(''));
let doc = Automerge.init(); 
const socket = new net.Socket();
const port = process.argv[2] || 3000;
let clock = 0n;


// Set up express server
const app = express();
app.use(express.json());

app.get('/', (_req, res) => {
  res.sendFile(__dirname + '/pages/index.html');
});

// Get all key-value pairs
app.get('/map', (_req, res) => {
    const docv = JSON.stringify(doc);
    console.log(docv);
    res.json(docv);
});

// Get value by key
app.get('/map/:key', (req, res) => {
    const key = req.params.key;
    if (doc.hasOwnProperty(key)) {
        res.json({ key, value: doc[key] });
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
    doc = Automerge.change(doc, d => {
        d[key] = value;
    });
    changes.push(Automerge.getLastLocalChange(doc));
    res.json({ message: 'Entry added/updated', key, value });
});

// Delete a key-value pair
app.delete('/map/:key', (req, res) => {
    const key = req.params.key;
    doc = Automerge.change(doc, d => {
        delete d[key];
    });
    changes.push(Automerge.getLastLocalChange(doc));
    res.json({ message: 'Entry deleted', key });
});

app.get('/save', (_req, res) => {
    // Get all changes from the Automerge document
    //console.log(changes.length)
    const ser_changes = msgpack.encode(changes);
    let message = new Message(0, clock, ser_changes);
    msgProc.enqueueCounter(clock, (_data) => {
        console.log("Saved Successfully");
    });
    clock += 1n;
    socket.write(message.serialize());

    changes = [];
    res.status(200).json();
});

app.get('/query', (_req, res) => {
    let message = new Message(1, clock, "");
    msgProc.enqueueCounter(clock, (data) => {
        const jsonString = data.toString("utf-8");
        const array = JSON.parse(jsonString);

        for (let ser_change of array) {
            let change_array = msgpack.decode(ser_change);
            [doc] = Automerge.applyChanges(doc, change_array);
        }
        console.log("Queried Changes(" + array.length + ") applied successfully");
    });
    clock += 1n;
    socket.write(message.serialize());

    res.status(200).json();
});

// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});
