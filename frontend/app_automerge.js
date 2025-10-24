import net      from "net";
import express  from 'express';
import path     from "path";
import * as Automerge from "@automerge/automerge";
import msgpack from "@msgpack/msgpack";

import * as SOCKET from "./src/socket.js";
import * as ENV from './src/env.js';

const __dirname = path.resolve(path.dirname(''));
let [doc] = Automerge.applyChanges(Automerge.init(), ENV.automerge_baseStateChange);
const app = express();
const port = process.argv[2] || 3000;
let counter = 0;
let changes = [];
console.log(ENV.state)

// Set up express server
SOCKET.init(20076, "127.0.0.1", port)
app.use(express.json());

app.get('/save', async function (req, res) {
    // Get all changes from the Automerge document
    if (ENV.debug){
        console.log("/save request")
    }
    const ser_changes = msgpack.encode(changes);
    const times = await SOCKET.save(ser_changes);

    changes = [];
    res.status(200).json();
});

app.get('/query', async function (req, res)  {

    const updateCursorHeader = req.headers['update_cursor'];
    const updateCursor = updateCursorHeader === 'true' || updateCursorHeader === '1';

    if (ENV.debug){
        console.log("/query request")
    }

    const times = await SOCKET.query((buffer) => {
        let change_buffer = msgpack.decode(buffer);
        if (ENV.debug){
            console.log("doc:", doc);
            console.log("change_buffer:", change_buffer);
        }
        [doc] = Automerge.applyChanges(doc,change_buffer);
        //console.log("after_changes", Automerge.toJS(doc));
    }, updateCursor);

    res.status(200).json();
});


// Get all key-value pairs
app.get('/map', (_req, res) => {
    try{
        const docv = JSON.stringify(doc);
        console.log(docv);
        res.json(docv);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Get value by key
app.get('/map/:key', (req, res) => {
    try{
        const key = req.params.key;

        if (doc.hasOwnProperty(key)) {
            res.json({ key, value: doc[key] });
        } else {
            res.status(404).json({ error: 'Key not found' });
        }
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Add or update key-value pair
app.post('/map', (req, res) => {
    try{
        if (ENV.debug){
            console.log("POST /map request")
        }

        const { key, value } = req.body;
        if (!key || value === undefined) {
            return res.status(400).json({ error: 'Key and value are required' });
        }
        doc = Automerge.change(doc, d => {
            d[key] = value;
        });        
        const change = Automerge.getLastLocalChange(doc);
        changes.push(change);

        res.json({ message: 'Entry added/updated', key, value });
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Delete a key-value pair
app.delete('/map/:key', (req, res) => {
    try{
        const key = req.params.key;
        doc = Automerge.change(doc, d => {
            delete d[key];
        });

        const change = Automerge.getLastLocalChange(doc);
        changes.push(change);

        res.json({ message: 'Entry deleted', key });
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});
