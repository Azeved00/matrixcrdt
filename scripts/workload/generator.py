import random
import sys
from math import ceil
from stats import * 

def select_pharmacy(n_pharmacies):
    return random.randrange(0, n_pharmacies)

def select_doctor(n_doctors):
    return random.randrange(0, n_doctors)

def select_both(n_pharmacies, n_doctors):
    if random.choice([True, False]):
        num = random.randrange(0, n_doctors)
        return ("D", num)
    else:
        num = random.randrange(0, n_pharmacies)
        return ("P", num)

def select_medication():
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

    i=0
    while i<operations:
        op = random.choices(
            population=list(OPERATIONS.keys()),
            weights=[op["prob"] for op in OPERATIONS.values()],
            k=1
        )[0]

        match OPERATIONS[op]["name"]:
            case "get_pharmacy_prescriptions":
                p= select_pharmacy(n_pharmacies)
                print(f"P {p} {OPERATIONS[op]["name"]} {p}")                      

            case "get_prescription_medication":
                if len(prescriptions) <= 0:
                    continue

                (who, idx) = select_both(n_pharmacies, n_doctors)
                presc = select_prescription(prescriptions)
                print(f"{who} {idx} {OPERATIONS[op]["name"]} {presc}")

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
                presc = select_prescription(prescriptions)
                precriptions.remove(prescription)

                print(f"P {idx} {OPERATIONS[op]["name"]} {presc}")                      

            case "update_prescription_medication":
                if len(prescriptions) <= 0:
                    continue

                idx = select_doctor(n_doctors)
                presc = select_prescription(prescriptions)
                medication = select_medication()

                print(f"D {idx} {OPERATIONS[op]["name"]} {presc}")                      
        i+=1

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <clients> <operations>")
    else:
        try:
            clients = int(sys.argv[1])
            pharmacies =ceil(clients * PHARMACIES)
            doctors = clients - pharmacies

            operations = int(sys.argv[2])

            gen_workload(doctors, pharmacies, operations)
        except ValueError:
            print("Both arguments must be integers.")
