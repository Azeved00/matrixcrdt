import random
from stats import * 
from numpy import random as nprandom


def zipf(max_value, a=1.5):
    while True:
        val = nprandom.zipf(a)
        if val <= max_value:
            return val-1

def select_pharmacy():
    return zipf(PHARMACIES, 1.5)

def select_doctor():
    return zipf(DOCTORS, 3)

def select_medication(MEDICINE):
    number = nprandom.poisson(lam=2)
    meds = set()
    for _ in range(number):
        meds.add(zipf(MEDICINE, 1.1))
    return ",".join(str(med) for med in meds)

def select_patient():
    return zipf(PATIENTS, 10)

def select_prescription(prescriptions):
    return random.choice(list(prescriptions))

def make_request(operation, path, data={}):
    match OPERATIONS[operation]["req"]:
            case "get":
                response = requests.get(path)
            case "post":
                response = requests.post(path, json=data)
        #print(response.json())
        return response.elapsed


def gen_workload(id, time):
    prescriptions = set()
    server_addr=calc_server_addr(id)
    for i in range(0, INITIAL_STATE_SIZE):
        prescriptions.add(99_000_00+i)
    prescription_id = INITIAL_STATE_SIZE
    
    log = open(f"{id}_log.csv", 'w')
    log.write("id,operation_name,elapsed")

    #print(f"{clients}")
    start_time = time.time()

    i=0
    while time.time() - start_time < seconds:
        op = random.choices(
            population=list(OPERATIONS.keys()),
            weights=[op["prob"] for op in OPERATIONS.values()],
            k=1
        )[0]

        match OPERATIONS[op]["name"]:
            case "get_pharmacy_prescriptions":
                p= select_pharmacy()
                path = OPERATIONS[op][path].format(pharmacy=p)

                elapsed = make_request(op, server_addr+path)

            case "get_prescription_medication":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to get the medication of")

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op][path].format(prescription=presc)
                elapsed = make_request(op,  server_addr+path)

            case "get_staff_prescription":
                d = select_doctor()
                path = OPERATIONS[op][path].format(doctor=d)

                elapsed = make_request(op,  server_addr+path)

            case "create_prescription":
                doc = select_doctor()
                patient = select_patient()
                pharmacy = select_pharmacy()

                prescriptions.add(prescription_id)
                prescription_id=prescription_id+1
                path = OPERATIONS[op][path]

                data = {
                    "patient": patient,
                    "doctor": doc,
                    "pharmacy":pharmacy,
                    "id": (prescription_id + (id*1_000_000)),
                }

                elapsed = make_request(op, server_addr+ path, data=data)


            case "get_processed_pharmacy_prescriptions":
                p = select_pharmacy()
                path = OPERATIONS[op][path].format(pharmacy=p)
                elapsed = make_request(op, server_addr+ path)

            case "process_prescription":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to process")

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op][path].format(prescription=presc)
                precriptions.remove(prescription)


            case "update_prescription_medication":
                if len(prescriptions) <= 0:
                    raise Exception("No Prescriptions to update")

                medication = select_medication()
                data = { "medication": medication}

                presc = select_prescription(prescriptions)
                path = OPERATIONS[op][path].format(prescription=presc)

                elapsed = make_request(op, server_addr+ path, data=data)

            case "get_patient":
                p = select_patient()
                path = OPERATIONS[op][path].format(patient=p)
                elapsed = make_request(op, server_addr+ path)

            case "get_prescription":
                presc = select_prescription(prescriptions)
                path = OPERATIONS[op][path].format(prescription=presc)
                elapsed = make_request(op, server_addr+ path)


        #print(i, " " ,log)
        log.write(f"{i}, {OPERATIONS[op]["name"]}, {elapsed}")
        i+=1

