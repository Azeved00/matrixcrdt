import express  from 'express';
import * as Automerge from "@automerge/automerge";
import * as SOCKET from "./src/socket.js";
import msgpack from "@msgpack/msgpack";

let doc = Automerge.from({
    prescriptionMap: {},
    staffMap : {},
    pharmacyMap: {},
    processedMap: {}
})
const app = express();
const port = process.argv[2] || 3000;
let counter = 0;
let changes = [];

SOCKET.init(20076, "127.0.0.1")

class Prescription {
    constructor(id, patient, doctor, pharmacy) {
        this.id = id;
        this.patient = patient;
        this.doctor = doctor;
        this.pharmacy = pharmacy;

        this.medication = [];
        this.processed = false;
    }
}

// Set up express server
app.use(express.json());
async function save(){
    const ser_changes = msgpack.encode(changes);
    changes = [];
    await SOCKET.save(ser_changes)
}

async function query() {
    await SOCKET.query((buffer) => {
        let change_array = msgpack.decode(buffer);
        [doc] = Automerge.applyChanges(doc, change_array);
    })
}


// get_pharmacy_prescriptions
app.get('/get_pharmacy_prescriptions/:pharmacy', async (req, res) => {
    try{
        const pharmacy = req.params.pharmacy;

        await query()
        const ret =doc["pharmacyMap"][pharmacy]; 
        //console.log(ret);

        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get prescription_medicine
app.get('/get_prescription_medication/:prescription', async (req, res) => {
    try {
        const prescription = req.params.prescription;


        await query();
        const ret =doc["prescriptionMap"][prescription].medication; 
        //console.log(ret);

        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get_staff prescription 
app.get('/get_staff_prescriptions/:doctor', async (req, res) => {
    try{
        const doctor = req.params.doctor;

        await query()
        const ret =doc["staffMap"][doctor]; 
        //console.log(ret);

        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// create prescription
app.post('/create_prescription', async  (req, res) => {
    try {
        const { patient, doctor, pharmacy} = req.body;
        let presc = new Prescription(counter, patient, doctor, pharmacy)
        console.log(presc)
        //presc.medication.push(params.medication)
        

        doc = Automerge.change(doc, (doc) => {
            doc.pharmacyMap[presc.pharmacy][presc.id] = true;
            doc.staffMap[presc.doctor][presc.id] = true;

            doc.prescriptionMap[presc.id] = presc;
        });

        counter += 1

        await save();
        res.status(200).json({});
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get processsed prescription 
app.get('/get_processed_pharmacy_prescriptions/:pharmacy', async (req, res) => {
    try{
        const pharmacy = req.params.pharmacy;
        await query();
        
        const processed = doc["processedMap"][pharmacy]
        console.log(processed)


        res.status(200).json(processed);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// process prescription
app.post('/process_prescrition', async (req, res) => {
    try{
        const params = req.body;
        
        doc = Automerge.change(doc, (doc) => {
            pharmacy = doc.prescriptionMap[params.prescription].pharmacy;
            doc.prescriptionMap[params.prescription].processed = true;

            doc.processedMap[pharmacy] ??= [];
            doc.processedMap[pharmacy].push(params.prescription)
        });
        
        await save();
        res.status(200).json({});
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// update prescription medicine
app.post('/update_prescription_medication', async (req, res) => {
    try {
        const params = req.body;

        dcrdt = Automerge.change(doc, (doc) => {
            doc.prescriptionMap[params.id] = params.medicine
        });

        await save();
        res.status(200).json();
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});
