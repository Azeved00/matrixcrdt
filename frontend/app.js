import net      from "net";
import express  from 'express';
import path     from "path";
import dcrdtLib from './lib/delta-crdt/frontend/index.js';

const __dirname = path.resolve(path.dirname(''))
let dcrdt = dcrdtLib.init({});
const client = new net.Socket();
const app = express();
const port = 3000;

// Set up socket to Auth Dag
//client.connect(20076, "127.0.0.1", () => {
    //console.log("Connected to Rust server!");
//});

client.on("data", (data) => {
    console.log("Received:", data.toString());
});

client.on("close", () => {
    console.log("Connection closed");
});


// Set up express server
app.use(express.json());

app.get('/', (req, res) => {
  res.sendFile(__dirname + '/index.html');
});

// Get all key-value pairs
app.get('/map', (req, res) => {
    const docv = JSON.stringify(dcrdtLib.documentValue(dcrdt), (key, value) => 
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

app.get('/save', (req, res) => {
    const delta = dcrdtLib.getChanges(dcrdt);
    res.status(200).json();
});

app.get('/query', (req, res) => {
    console.log("Query is not yet implemented")
});



// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});
