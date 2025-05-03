import express  from 'express';
import DCRDT from './lib/delta-crdt/frontend/index.js';
import DCRDT_ENCODER from './lib/delta-crdt/frontend/encoder.js';
import * as SOCKET from './src/socket.js'

const isDev = process.env.NODE_ENV === 'dev';

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
let counter = 0;

//socket set up
SOCKET.init(20076, "127.0.0.1")

async function query() {
    await SOCKET.query((buffer) => {
        let change = DCRDT_ENCODER.decode(buffer)
        if(isDev){
            //console.log(change)
        }
        DCRDT.applyChanges(dcrdt, change)
    });
}

async function save(){
    const delta = DCRDT.getChanges(dcrdt);
    const ser_delta = DCRDT_ENCODER.encode(delta);

    if(isDev) {
        //console.log(ser_delta)
    }

    await SOCKET.save(ser_delta)
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
app.get('/patients/:id', async (_req, res) => {
    try{
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

        await query()
        const val = DCRDT.documentValue(dcrdt);
        const ret = Object.keys(val.pharmacyMap[pharmacy] ?? []);
        if(isDev) {
            console.log(ret);
        }

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
        await query();

        const val = DCRDT.documentValue(dcrdt)
        const list = Object.keys(val.processedMap[pharmacy] ?? [])

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

        await query();
        const val = DCRDT.documentValue(dcrdt);
        const ret = Object.keys(val.staffMap[doctor] ?? []);
        if(isDev) {
            console.log(ret);
        }

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

        await query();
        const val = DCRDT.documentValue(dcrdt);
        const ret =val.prescriptionMap[prescription];
        //TODO if prescription does not exist then make ERROR
        if(isDev) {
            console.log(ret);
        }

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
        let presc = prescription(id, patient, doctor, pharmacy)
        //console.log(presc)
        
        const val = DCRDT.documentValue(dcrdt)
        if(isDev){
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
        const params = req.body;

        const val = DCRDT.documentValue(dcrdt)
        const presc = val.prescriptionMap[params.prescription]
        if(!(presc.pharmacy in val.processed)){
            dcrdt = DCRDT.change(dcrdt, "", (doc) => {
                doc.processed[presc.pharmacy] = {} 
            })
        }
        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.prescriptionMap[presc.prescription].processed = true;
            doc.processedMap[params.prescription] = true;
        });
        
        await save();
        res.status(200).json();
    } catch (err) {
        res.status(500).send({ error: err.toString() });
    }
});

// get prescription_medicine
app.get('/prescription/:prescription/medicine', async (req, res) => {
    try {
        const prescription = req.params.prescription;

        await query();
        const val = DCRDT.documentValue(dcrdt);
        const ret =val.prescriptionMap[prescription].medication;
        if(isDev) {
            console.log(ret);
        }

        res.status(200).json(ret);
    } catch (err) {
        console.log(err)
        res.status(500).send({ error: err.toString() });
    }
});


// update prescription medicine
app.post('/prescription/:prescription/medicine', async (req, res) => {
    try {
        const params = req.body;

        dcrdt = DCRDT.change(dcrdt, "", (doc) => {
            doc.prescriptionMap[params.id] = params.medicine
        });

        await save();
        res.status(200).json();
    } catch (err) {
        res.status(500).send({ error: err.toString() });
    }
});

// Start the server
app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});

