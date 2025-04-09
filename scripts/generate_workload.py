import random
import string
import sys

PHARMACIES=100
DOCTROS = 100
OPERATIONS = 10 000


OPERATIONS = [
        {
          "index": 0,
          "name": 'get_pharmacy_prescriptions', 
          "prob": 0.27,
          "params": ['pharmacy'],
        },
        {
          "index": 1,
          "name": 'get_prescription_medication', 
          "prob": 0.27,
          "params": ['prescription'],
        },
        {
          "index": 2,
          "name": 'get_staff_prescription', 
          "prob": 0.14,
          "params": ['doctor'],
        },
        {
          "index": 3,
          "name": 'create_prescription', 
          "prob": 0.08,
          "params": ['patient', 'doctor', 'pharmacy'],
        },
        {
          "index": 4,
          "name": 'get_processed_pharmacy_prescription', 
          "prob": 0.07,
          "params": ['pharmacy']
        },
        {
          "index": 5,
          "name": 'process_prescription', 
          "prob": 0.04,
          "params": ['prescription']
        },
        {
          "index": 6,
          "name": 'update_prescription_medication', 
          "prob": 0.04,
          "params": ['prescription', 'medication']
        }]


def __main__:
    doctors = []*DOCTORS
    pharmacies = []*PHARMACIES
    prescriptions = set()
    prescription_id = 0
    
    #bootstrap operations
    #just so that getting editing or closing prescriptions has targets

    for i in range(0,OPERATIONS):
        let op = random.choices([op["index"] for op in OPERATIONS], weights=[op["prob"] for op in OPERATIONS], k=1)

        for param in OPERATIONS[op]["params"]:
            match param:
                case "doctor":
                    let doctor = random.randint(0, len(doctors))
                case "pharmacy":
                    #chose on eof the pharmacies
                    let pharmacy =  random.randint(0, len(doctors))
                case "patient":
                    # random number
                    let patient = random.randint(1, 1000)
                case "prescription":
                    #chose one of the prescriptions
                    let new_prescription = random.choices(prescriptions, k=1)
                case "medication":
                    let medication = random.randint(1, 1000)
                case _ :
                    print("ERROR")

        #print into the file

        #change prescription accordingly
        match op:
            #create_prescriton creates a new one
            case 3:
                prescriptions.add(prescription_id)
                prescription_id=prescription_id+1
            
            #process prescription removes from the prescription array
            case 5:
                precriptions.remove(new_prescription)
                

        
