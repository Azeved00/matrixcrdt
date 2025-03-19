import json
import random
import string
import sys

def random_path():
    return f"/{random.choice(['name', 'age', 'city', 'email', 'score', 'status'])}"

def random_value():
    value_types = ["string", "int", "bool"]
    value_type = random.choice(value_types)

    if value_type == "string":
        return "".join(random.choices(string.ascii_letters, k=5))
    elif value_type == "int":
        return random.randint(1, 100)
    elif value_type == "bool":
        return random.choice([True, False]) 

# Function to generate random change operations
def generate_changes(num_operations):
    changes = []
    operations = ["add", "replace", "remove"]

    for _ in range(num_operations):
        op = random.choice(operations) 
        path = random_path() 
        value = None

        if op in ["add", "replace"]:
            value = random_value()  
        
        change = {"op": op, "path": path}
        if value is not None:
            change["value"] = value
        
        changes.append(change)
    
    return changes

if len(sys.argv) != 2:
    print("Usage: python3 script.py <num_operations>")
    sys.exit(1)

try:
    num_operations = int(sys.argv[1])
except ValueError:
    print("Error: <num_operations> must be an integer.")
    sys.exit(1)

changes = generate_changes(num_operations)
with open("changes.json", "w") as f:
    json.dump(changes, f, indent=4)

print(f"Generated 'changes.json' with {num_operations} random changes.")
