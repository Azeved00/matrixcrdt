PHARMACIES= 0.065
DOCTORS = 0.935
OPERATIONS = 10_000
PATIENTS = 1_000_000
MEDICINE = 10_000

OPERATIONS = {
    'get_pharmacy_prescriptions':{
      "index": 0,
      "name": 'get_pharmacy_prescriptions', 
      "prob": 0.27,
      "params": ['pharmacy'],
      "who":'pharmacy',
      "req":"get",
    },
    'get_prescription_medication':{
      "index": 1,
      "name": 'get_prescription_medication', 
      "prob": 0.27,
      "params": ['prescription'],
      "who": 'both',
      "req":"get",
    },
    'get_staff_prescriptions':{
      "index": 2,
      "name": 'get_staff_prescriptions', 
      "prob": 0.14,
      "params": ['doctor'],
      "who": 'doctor',
      "req":"get",
    },
    'create_prescription':{
      "index": 3,
      "name": 'create_prescription', 
      "prob": 0.08,
      "params": ['patient', 'doctor', 'pharmacy'],
      "who": 'doctor',
      "req": "post",
    },
    'get_processed_pharmacy_prescriptions':{
      "index": 4,
      "name": 'get_processed_pharmacy_prescriptions', 
      "prob": 0.07,
      "params": ['pharmacy'],
      "who": 'pharmacy',
      "req":"get",
    },
    'process_prescription':{
      "index": 5,
      "name": 'process_prescription', 
      "prob": 0.04,
      "params": ['prescription'],
      "who": 'pharmacy',
      "req": "post",
    },
    'update_prescription_medication':{
      "index": 6,
      "name": 'update_prescription_medication', 
      "prob": 0.04,
      "params": ['prescription', 'medication'],
      "who": 'doctor',
      "req": "post",
    }
}
