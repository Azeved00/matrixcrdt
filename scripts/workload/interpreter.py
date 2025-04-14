import sys
from stats import *
import requests

def parse_file(filename):
    with open(filename, 'r') as f:
        lines = f.readlines()

    # Parse number of doctors and pharmacies
    num_pharmacies, num_doctors = map(int, lines[0].split())
    
    operations = []
    
    # Parse each operation line
    for line in lines[1:]:
        parts = line.strip().split()
        entity_type = parts[0]         # 'D' or 'P'
        entity_id = int(parts[1])      # index of the doctor/pharmacy
        operation = parts[2]           # operation name

        parameters = list(map(int, parts[3:]))  # the rest are parameters (as integers)


        print(operation, parameters)
        match OPERATIONS[operation]["req"]:
            case "get":
                p_name = OPERATIONS[operation]["params"][0]
                p = parameters[0]
                response = requests.get(f"http://localhost:3000/{operation}/{p}")

            case "post":
                parameter_names = OPERATIONS[operation]["params"]
                data = dict(zip(parameter_names, parameters))
                print(data)
                response = requests.post(f"http://localhost:3000/{operation}", json=data)
        print(response.json())

        operations.append({
            'type': entity_type,
            'id': entity_id,
            'op': operation,
            'params': data
        })

    return num_pharmacies, num_doctors, operations

if __name__ == "__main__":

    if len(sys.argv) != 2:
        print("Usage: python script.py <file>")
    else:
        file = sys.argv[1]
        pharmacies, doctors, operations = parse_file(file)
        #print(operations)
