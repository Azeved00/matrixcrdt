import express  from 'express';
import * as Automerge from "@automerge/automerge";
import * as SOCKET from "./src/socket.js";
import msgpack from "@msgpack/msgpack";
import { Logger } from './src/logger.js';
import * as ENV from './src/env.js';

let [doc] = Automerge.applyChanges(Automerge.init(), ENV.automerge_baseStateChange);
const app = express();
const port = process.argv[2] || 3000;
const logger = new Logger(`log_${port}.csv`);
let counter = 0;

SOCKET.init(20076, "127.0.0.1", port)

class Prescription {
    constructor(id, patient, doctor, pharmacy) {
        this.id = id;
        this.patient = patient;
        this.doctor = doctor;
        this.pharmacy = pharmacy;

        this.medication = "";
        this.processed = false;
    }
}

// Set up express server
app.use(express.json());
async function save(change){
    const ser_change = msgpack.encode(change);
    const times = await SOCKET.save(ser_change);
    return times;
}

async function query() {
    const times = await SOCKET.query((buffer) => {
        let change_buffer = msgpack.decode(buffer);
        [doc] = Automerge.applyChanges(doc,[new Uint8Array(change_buffer)]);
        //console.log("after_changes", Automerge.toJS(doc));
    });
    return times;
}
//-------------------------------------------------------
// PATIENTS
//-------------------------------------------------------
//get patient
app.get('/patient/:patient', async (req, res) => {
    try{
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();
        const query_times = await query();

        if (ENV.debug){
            console.log(req.params.patient)
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;
        logger.log(requestId, SOCKET.clock, "get_patient", time);
        res.status(200).json({});
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }

});

//-------------------------------------------------------
// PHARMACY
//-------------------------------------------------------
// get_pharmacy_prescriptions
app.get('/pharmacy/:pharmacy/prescriptions', async (req, res) => {
    try{
        const pharmacy = req.params.pharmacy;
        const requestId = req.headers['request-id'] || -1;
        const query_times = await query();
        const start = process.hrtime();

        const ret = doc["pharmacyMap"][pharmacy]; 
        //console.log(ret);

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;
        logger.log(requestId, SOCKET.clock, "get_pharmacy_prescriptions", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get processsed prescription 
app.get('/pharmacy/:pharmacy/processed', async (req, res) => {
    try{
        const pharmacy = req.params.pharmacy;
        const requestId = req.headers['request-id'] || -1;
        const query_times = await query();
        const start = process.hrtime();
        
        const processed = doc["processedMap"][pharmacy]
        if(ENV.debug){
            console.log(processed)
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;
        logger.log(requestId, SOCKET.clock, "get_processed_prescription", time);
        res.status(200).json(processed);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

//-------------------------------------------------------
// DOCTOR
//-------------------------------------------------------
// get_staff prescription 
app.get('/staff/:doctor/prescriptions', async (req, res) => {
    try{
        const doctor = req.params.doctor;
        const requestId = req.headers['request-id'] || -1;
        const query_times = await query();
        const start = process.hrtime();

        const ret =doc["staffMap"][doctor]; 
        //console.log(ret);

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;;
        logger.log(requestId, SOCKET.clock, "get_staff_prescription", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

//-------------------------------------------------------
// PRESCRIPTIONS
//-------------------------------------------------------
////get prescription
app.get('/prescription/:prescription', async (req, res) =>{
    try {
        const prescription = req.params.prescription;
        const requestId = req.headers['request-id'] || -1;
        const query_times = await query();
        const start = process.hrtime();

        const ret = doc.prescriptionMap[prescription];
        //TODO if prescription does not exist then make ERROR
        if(ENV.debug) {
            console.log(ret);
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;
        logger.log(requestId, SOCKET.clock, "get_prescription", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// create prescription
app.post('/prescription', async  (req, res) => {
    try {
        const { patient, doctor, pharmacy, id} = req.body;
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();
        let presc = new Prescription(id, patient, doctor, pharmacy)

        if(ENV.debug){
            console.log("create prescription")
            console.log(presc)
            console.log(doc)
        }
        doc = Automerge.change(doc, (doc) => {
            doc.pharmacyMap[presc.pharmacy] ??= [];
            doc.pharmacyMap[presc.pharmacy].push(presc.id);

            doc.staffMap[presc.doctor] ??= [];
            doc.staffMap[presc.doctor].push(presc.id);

            doc.prescriptionMap[presc.id] = presc;
        });
        const change = Automerge.getLastLocalChange(doc);

        counter += 1
        await save(change);
        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "create_prescription", time);
        res.status(200).json({});
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// process prescription
app.post('/prescription/:prescription/process', async (req, res) => {
    try{
        const prescription = req.params.prescription;
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();
        
        const presc = doc.prescriptionMap[prescription];
        if(ENV.debug){
            console.log(doc.prescriptionMap)
            console.log(presc)
        }
        if (!presc || Object.keys(presc).length===0){
            console.log(presc)
            throw new Error(`Prescription ${prescription} Not found in document`);

        }
        doc = Automerge.change(doc, (doc) => {
            doc.prescriptionMap[prescription].processed = true;

            doc.processedMap[presc.pharmacy] ??= [];
            doc.processedMap[presc.pharmacy].push(prescription)
        });
        const change = Automerge.getLastLocalChange(doc);
        
        await save(change);
        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "process_prescription", time);
        res.status(200).json({});
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

//-------------------------------------------------------
// PRESCRIPTION MEDICATION
//-------------------------------------------------------

// get prescription_medicine
app.get('/prescription/:prescription/medication', async (req, res) => {
    try {
        const prescription = req.params.prescription;
        const requestId = req.headers['request-id'] || -1;
        const query_times = await query();
        const start = process.hrtime();


        const presc =doc.prescriptionMap[prescription] ??= {} 
        if (ENV.debug) {
            console.log(presc)
        }
        const ret = presc.medication

        //console.log(ret);

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3) + query_times.state_apply;
        logger.log(requestId, SOCKET.clock, "get_prescription_medication", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// update prescription medicine
app.post('/prescription/:prescription/medication', async (req, res) => {
    try {
        const prescription = req.params.prescription;
        const { medication } = req.body;
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();

        if(ENV.debug){
            console.log("UPDATE PRESCRIPTION MEDICINE")
            console.log(prescription)
            console.log(doc.prescriptionMap)
            console.log(medication)
        }
        doc = Automerge.change(doc, (doc) => {
            doc.prescriptionMap[prescription].medication = medication
        });
        const change =Automerge.getLastLocalChange(doc)

        await save(change);
        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock,"update_prescription_medication", time);
        res.status(200).json();
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// Start the server
app.listen(port, () => {
    if (ENV.debug) {
        console.log(`Server running at http://localhost:${port}`);
    }
});
