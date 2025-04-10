import random
import string
import sys

PHARMACIES= 0.065
DOCTORS = 0.935
OPERATIONS = 10_000
PATIENTS = 1_000_000
MEDICINE = 10_000

OPERATIONS = [
        {
          "index": 0,
          "name": 'get_pharmacy_prescriptions', 
          "prob": 0.27,
          "params": ['pharmacy'],
          "who":'pharmacy',
        },
        {
          "index": 1,
          "name": 'get_prescription_medication', 
          "prob": 0.27,
          "params": ['prescription'],
          "who": 'both',
        },
        {
          "index": 2,
          "name": 'get_staff_prescription', 
          "prob": 0.14,
          "params": ['doctor'],
          "who": 'doctor',
        },
        {
          "index": 3,
          "name": 'create_prescription', 
          "prob": 0.08,
          "params": ['patient', 'doctor', 'pharmacy'],
          "who": 'doctor',
        },
        {
          "index": 4,
          "name": 'get_processed_pharmacy_prescription', 
          "prob": 0.07,
          "params": ['pharmacy'],
          "who": 'pharmacy',
        },
        {
          "index": 5,
          "name": 'process_prescription', 
          "prob": 0.04,
          "params": ['prescription'],
          "who": 'pharmacy',
        },
        {
          "index": 6,
          "name": 'update_prescription_medication', 
          "prob": 0.04,
          "params": ['prescription', 'medication'],
          "who": 'doctor',
        }]


def select_pharmacy(n_pharmacies):
    return random.randint(0, n_pharmacies)

def select_doctor(n_doctors):
    return random.randint(0, n_doctors)

def select_both(n_pharmacies, n_doctors):
    if random.choice([True, False]):
        num = random.randint(1, n_doctors)
        return ("D", num)
    else:
        num = random.randint(1, n_pharmacies)
        return ("P", num)

def select_medicine():
    return random.randint(1, MEDICINE)

def select_patient():
    return random.randint(1, PATIENTS)

def select_prescription(prescriptions):
    return random.choice(list(prescriptions))


def gen_workload(n_doctors, n_pharmacies, operations):
    doctors = []*n_doctors
    pharmacies = []*n_pharmacies
    prescriptions = set()
    prescription_id = 0
    
    print(f"{n_doctors} {n_pharmacies}")

    #bootstrap operations
    #just so that getting editing or closing prescriptions has targets
    i=0
    while i<operations:
        op = random.choices([op["index"] for op in OPERATIONS], weights=[op["prob"] for op in OPERATIONS], k=1)[0]

        match OPERATIONS[op]["name"]:
            case "get_pharmacy_prescriptions":
                p= select_pharmacy(n_pharmacies)
                print(f"P {p} {OPERATIONS[op]["name"]} {p}")                      

            case "get_prescription_medication":
                if len(prescriptions) <= 0:
                    continue

                (who, idx) = select_both(n_pharmacies, n_doctors)
                presc = select_prescription(prescriptions)
                print(f"{who} {idx} {OPERATIONS[op]["name"]} {prec}")

            case "get_staff_prescription":
                idx = select_doctor(n_doctors)
                print(f"D {idx} {OPERATIONS[op]["name"]} {idx}")


            case "create_prescription":
                idx = select_doctor(n_doctors)
                
                patient = select_patient()
                pharmacy = select_pharmacy(n_pharmacies)

                prescriptions.add(prescription_id)
                prescription_id=prescription_id+1

                print(f"D {idx} {OPERATIONS[op]["name"]} {patient} {idx} {pharmacy}")


            case "get_processed_pharmacy_prescriptions":
                idx = select_pharmacy(n_pharmacies)
                print(f"P {idx} {OPERATIONS[op]["name"]} {idx}")                      

            case "process_prescription":
                if len(prescriptions) <= 0:
                    continue

                idx = select_pharmacy(n_pharmacies)
                prescription = select_prescription(prescriptions)
                precriptions.remove(prescription)

                print(f"P {idx} {OPERATIONS[op]["name"]} {prescripton}")                      

            case "update_prescription_medication":
                if len(prescriptions) <= 0:
                    continue

                idx = select_doctor(n_doctors)
                prescription = select_prescription(prescriptions)
                medication = select_medication()

                print(f"D {idx} {OPERATIONS[op]["name"]} {prescripton}")                      
        i+=1

if __name__ == "__main__":
    random.seed(42)

    if len(sys.argv) != 3:
        print("Usage: python script.py <clients> <operations>")
    else:
        try:
            clients = int(sys.argv[1])
            pharmacies =round(clients * PHARMACIES)
            doctors = clients - pharmacies

            operations = int(sys.argv[2])

            gen_workload(doctors, pharmacies, operations)
        except ValueError:
            print("Both arguments must be integers.")
