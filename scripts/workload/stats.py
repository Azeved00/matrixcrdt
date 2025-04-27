
# OTHER CONSTANTS
CLIENT_BASE_PORT=3030
INITIAL_STATE_SIZE=10
def calc_server_addr(client_id):
    return "http://localhost:" + str(client_id)

# FMKE STATS
PHARMACIES= 300
DOCTORS = 5_000
PATIENTS = 1_000_000
MEDICINE = 10_000
HOSPITALS = 50

# FMKE OPERATIONS
OPERATIONS = {
    'get_pharmacy_prescriptions':{
      "index": 0,
      "name": 'get_pharmacy_prescriptions', 
      "prob": 0.27,
      "params": ['pharmacy'],
      "req":"get",
      "path":"/pharmacy/{pharmacy}/prescriptions",
    },
    'get_prescription_medication':{
      "index": 1,
      "name": 'get_prescription_medication', 
      "prob": 0.27,
      "params": ['prescription'],
      "req":"get",
      "path":"/prescription/{prescription}/medication",
    },
    'get_staff_prescriptions':{
      "index": 2,
      "name": 'get_staff_prescriptions', 
      "prob": 0.14,
      "params": ['doctor'],
      "req":"get",
      "path":"/staff/{doctor}/prescriptions",
    },
    'create_prescription':{
      "index": 3,
      "name": 'create_prescription', 
      "prob": 0.08,
      "params": ['patient', 'doctor', 'pharmacy'],
      "req": "post",
      "path":"/prescription",
    },
    'get_processed_pharmacy_prescriptions':{
      "index": 4,
      "name": 'get_processed_pharmacy_prescriptions', 
      "prob": 0.07,
      "params": ['pharmacy'],
      "req":"get",
      "path":"/pharmacy/{doctor}/processed",
    },
    'process_prescription':{
      "index": 5,
      "name": 'process_prescription', 
      "prob": 0.04,
      "params": ['prescription'],
      "req": "post",
      "path":"/prescription/{prescription}/process",
    },
    'update_prescription_medication':{
      "index": 6,
      "name": 'update_prescription_medication', 
      "prob": 0.04,
      "params": ['prescription', 'medication'],
      "req": "post",
      "path":"/prescription/{prescription}/medication",
    },
    'get_patient':{
      "index": 7,
      "name": 'get_patient', 
      "prob": 0.05,
      "params": ['patient'],
      "req": "get",
      "path":"/patient/{patient}",
    },
    'get_prescription':{
      "index": 8,
      "name": 'get_prescription', 
      "prob": 0.05,
      "params": ['prescription'],
      "req": "get",
      "path":"/prescription/{prescription}",
    }
}
