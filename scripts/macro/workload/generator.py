from .stats import * 
from numpy import random as nprandom
import time as timelib
import requests
import sys
from tqdm import tqdm


def zipf(max_value, a=1.5, rng=None):
    if rng is None:
        rng = nprandom.default_rng()

    while True:
        val = rng.zipf(a)
        if val <= max_value:
            return val-1

def select_pharmacy():
    return zipf(PHARMACIES, 1.5)

def select_doctor():
    return zipf(DOCTORS, 3)

def select_medication(rng):
    if rng is None:
        rng = nprandom.default_rng()
    first_med = rng.integers(low=0, high=MEDICINE)
    meds = {first_med}

    additional_count = rng.poisson(lam=2)
    while len(meds) < additional_count + 1:
        med = zipf(MEDICINE, 1.1)
        meds.add(med)

    return ",".join(str(med) for med in meds)

def select_patient():
    return zipf(PATIENTS, 10)

def select_prescription(prescriptions, rng=None):
    if rng is None:
        rng = nprandom.default_rng()

    return rng.choice(list(prescriptions))

def make_request(operation, path,counter,  data={}):
    headers = {}
    headers['request-id'] = f"{counter}"

    start = timelib.time_ns()
    match OPERATIONS[operation]["req"]:
            case "get":
                response = requests.get(path, headers=headers)
            case "post":
                response = requests.post(path, json=data, headers=headers)
    end = timelib.time_ns()
    
    #print(response.json())

    if not response.ok:
        try:
            error_message = response.json().get("error", "Unknown error")
        except ValueError:
            error_message = response.text or "Unknown error (non-JSON response)"

        raise Exception(f"HTTP {response.status_code}: {error_message}")

    return (end-start) / 1000

def make_prescription_state(initial_size=10, id=0, rng=None):
    prescriptions = set(99_000_000 + i for i in range(initial_size))
    prescription_id = initial_size
    server_addr=calc_server_addr(id)

    def match_operation(op, counter, rng):
        nonlocal prescription_id
        elapsed = -1
        match OPERATIONS[op]["name"]:
            case "get_pharmacy_prescriptions":
                p= select_pharmacy()
                path = OPERATIONS[op]["path"].format(pharmacy=p)

                elapsed = make_request(op, server_addr+path, counter)

            case "get_prescription_medication":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to get the medication of")

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op]["path"].format(prescription=presc)
                elapsed = make_request(op,  server_addr+path, counter)

            case "get_staff_prescriptions":
                d = select_doctor()
                path = OPERATIONS[op]["path"].format(doctor=d)

                elapsed = make_request(op,  server_addr+path, counter)

            case "create_prescription":
                doc = select_doctor()
                patient = select_patient()
                pharmacy = select_pharmacy()

                presc = (prescription_id + (id*1_000))
                prescription_id=prescription_id+1
                prescriptions.add(presc)
                path = OPERATIONS[op]["path"]

                data = {
                    "patient": patient,
                    "doctor": doc,
                    "pharmacy":pharmacy,
                    "id": presc,
                }

                elapsed = make_request(op, server_addr+ path, counter, data=data)

            case "get_processed_pharmacy_prescriptions":
                p = select_pharmacy()
                path = OPERATIONS[op]["path"].format(pharmacy=p)
                elapsed = make_request(op, server_addr+ path, counter)

            case "process_prescription":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to process")

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op]["path"].format(prescription=presc)
                elapsed = make_request(op, server_addr+ path, counter, data={})
                prescriptions.remove(presc)


            case "update_prescription_medication":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to update")

                medication = select_medication(rng)
                data = { "medication": medication }

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op]["path"].format(prescription=presc)

                elapsed = make_request(op, server_addr+ path, counter, data=data)

            case "get_patient":
                p = select_patient()
                path = OPERATIONS[op]["path"].format(patient=p)
                elapsed = make_request(op, server_addr+ path, counter)

            case "get_prescription":
                presc = select_prescription(prescriptions)
                path = OPERATIONS[op]["path"].format(prescription=presc)
                elapsed = make_request(op, server_addr+ path, counter)
        return elapsed

    return match_operation

def gen_workload(id, number, rng=None):
    if rng is None:
        rng = nprandom.default_rng()

    bar = tqdm(total=number, desc=f"Thread {id}", position=id, leave=True)
    
    log = open(f"log_{id+1}.csv", 'w')
    log.write("id,operation_name,elapsed\n")

    #print(f"{clients}")
    match_operation = make_prescription_state(INITIAL_STATE_SIZE, id)
    start_time = timelib.time()

    counter=0
    while counter<number:
        op = rng.choice(
            list(OPERATIONS.keys()),
            p=[op["prob"] for op in OPERATIONS.values()],
        )

        elapsed = match_operation(op, counter, rng)

        #print(i, " " ,log)
        log.write(f"{counter}, {OPERATIONS[op]["name"]}, {elapsed}\n")
        counter+=1
        bar.update(1)
    bar.close()
