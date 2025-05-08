import express  from 'express';
import DCRDT from './lib/delta-crdt/frontend/index.js';
import DCRDT_ENCODER from './lib/delta-crdt/frontend/encoder.js';
import * as SOCKET from './src/socket.js'
import { Logger } from './src/logger.js';
import * as ENV from './src/env.js';

let dcrdt = DCRDT.init({})
dcrdt = DCRDT.change(dcrdt, "", (doc) => {
    doc.pharmacyMap = {}
})
dcrdt = DCRDT.change(dcrdt, "", (doc) => {
    doc.staffMap = {}
})
dcrdt = DCRDT.change(dcrdt, "", (doc) => {
    doc.prescriptionMap = {}
})
dcrdt = DCRDT.change(dcrdt, "", (doc) => {
    doc.processedMap = {}
})

const app = express();
app.use(express.json());
const port = process.argv[2] || 3000;
const logger = new Logger(`log_${port}.csv`);
let counter = 0;

//socket set up
SOCKET.init(20076, "127.0.0.1", port)

async function query() {
    await SOCKET.query((buffer) => {
        let change = DCRDT_ENCODER.decode(buffer)
        if(ENV.debug){
            //console.log(change)
            console.log("dcrdt before applying changes")
            const val1 = DCRDT.documentValue(dcrdt);
            console.log(val1)
        }

        DCRDT.applyChanges(dcrdt, change)
        if(ENV.debug){
            console.log("dcrdt after applying changes")
            const val2 = DCRDT.documentValue(dcrdt);
            console.log(val2)
        }
    });
}

async function save(){
    const delta = DCRDT.getChanges(dcrdt);
    const ser_delta = DCRDT_ENCODER.encode(delta);

    if(ENV.debug) {
        //console.log(ser_delta)
    }

    await SOCKET.save(ser_delta)
}

function from_set_prescription(set_json){
    const output = Object.fromEntries(
      Object.entries(set_json).map(([key, valueSet]) => 
          [key, valueSet.values().next().value]
      )
    );

    return output;
}

function prescription(id, patient, staff, pharmacy){
    return {
        id : id,
        patient : patient,
        staff : staff,
        pharmacy : pharmacy,

        medication : "",
        processed : false
    }
}

//-------------------------------------------------------
// PATIENTS
//-------------------------------------------------------
//get patient
app.get('/patient/:patient', async (req, res) => {
    try{
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();

        if (ENV.debug){
            console.log(req.params.patient)
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
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
        await query()
        const start = process.hrtime();

        const val = DCRDT.documentValue(dcrdt);
        const ret = Object.keys(val.pharmacyMap[pharmacy] ?? []);
        if(ENV.debug) {
            console.log(ret);
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "get_pharmacy_prescriptions", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get processsed prescription 
app.get('/pharmacy/:pharmacy/processed', async (req, res) => {
    try {
        const pharmacy = req.params.pharmacy;
        const requestId = req.headers['request-id'] || -1;
        await query();

        const start = process.hrtime();
        const val = DCRDT.documentValue(dcrdt)
        const list = Object.keys(val.processedMap[pharmacy] ?? [])

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "get_processed_prescription", time);
        res.status(200).json(list);
    } catch (err) {
        res.status(500).send({ error: err.toString() });
    }

    //get processed prescription from ProcessedPrescriptions array
});

//-------------------------------------------------------
// DOCTOR
//-------------------------------------------------------
// get_staff prescription 
app.get('/staff/:doctor/prescriptions', async (req, res) => {
    try{
        const doctor = req.params.doctor;
        const requestId = req.headers['request-id'] || -1;
        await query();
        const start = process.hrtime();

        const val = DCRDT.documentValue(dcrdt);
        const ret = Object.keys(val.staffMap[doctor] ?? []);
        if(ENV.debug) {
            console.log(ret);
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
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
//get prescription
app.get('/prescription/:prescription', async (req, res) =>{
    try {
        const prescription = req.params.prescription;
        const requestId = req.headers['request-id'] || -1;
        await query();
        const start = process.hrtime();

        const val = DCRDT.documentValue(dcrdt);
        const ret =val.prescriptionMap[prescription];
        //TODO if prescription does not exist then make ERROR
        if(ENV.debug) {
            console.log(ret);
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
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

        let presc = prescription(id, patient, doctor, pharmacy)
        if(ENV.debug){
            console.log(presc)
        }
        
        const val = DCRDT.documentValue(dcrdt)
        if(ENV.debug){
            console.log(val)
        }   
        if(!(presc.staff in val.staffMap)){
            dcrdt = DCRDT.change(dcrdt, "", (doc) => {
                doc.staffMap[presc.staff] = {} 
            })
        }
        if(!(presc.pharmacy in val.pharmacyMap)){
            dcrdt = DCRDT.change(dcrdt, "", (doc) => {
                doc.pharmacyMap[presc.pharmacy] = {} 
            })
        }
        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.staffMap[presc.staff][presc.id] = true 
            doc.pharmacyMap[presc.pharmacy][presc.id] = true
            doc.prescriptionMap[presc.id] = presc
        });
        counter += 1

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "create_prescription", time);
        await save();
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
        
        if(ENV.debug){
            console.log("process prescription")
        }

        const val = DCRDT.documentValue(dcrdt)
        const presc = from_set_prescription(val.prescriptionMap[prescription])
        if(ENV.debug){
            console.log(presc)
            console.log(val.processedMap)
        }

        if(!(presc.pharmacy in val.processedMap)){
            dcrdt = DCRDT.change(dcrdt, "", (doc) => {
                doc.processedMap[presc.pharmacy] = {} 
            })
        }
        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.prescriptionMap[prescription].processed = true;
        });
        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.processedMap[presc.pharmacy][prescription] = true;
        });
        
        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "process_prescription", time);
        await save();
        res.status(200).json();
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});

// get prescription_medication
app.get('/prescription/:prescription/medication', async (req, res) => {
    try {
        const prescription = req.params.prescription;
        const requestId = req.headers['request-id'] || -1;
        await query();
        const start = process.hrtime();

        const val = DCRDT.documentValue(dcrdt);
        console.log("get_prescription_meds")
        console.log(val)
        console.log(prescription)
        const ret =val.prescriptionMap[prescription].medication;

        if(ENV.debug) {
            console.log(ret);
        }

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock, "get_prescription_medication", time);
        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});


// update prescription medication
app.post('/prescription/:prescription/medication', async (req, res) => {
    try {
        const prescription = req.params.prescription;
        const { medication } = req.body;
        const requestId = req.headers['request-id'] || -1;
        const start = process.hrtime();

        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.prescriptionMap[prescription].medication = medication
        });

        const diff = process.hrtime(start);
        const time = (diff[0] * 1e6 + diff[1] / 1e3).toFixed(3);
        logger.log(requestId, SOCKET.clock,"update_prescription_medication", time);
        await save();
        res.status(200).json();
    } catch (err) {
        res.status(500).send({ error: err.toString() });
    }
});

// Start the server
app.listen(port, () => {
    if(ENV.debug){
        console.log(`Server running at http://localhost:${port}`);
    }
});

