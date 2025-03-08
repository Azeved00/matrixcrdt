import matplotlib.pyplot as plt

# Function to read the log file and extract the data
def extract_data_from_log(file_path):
    ids = []
    times = []
    
    with open(file_path, 'r') as file:
        for line in file:
            parts = line.strip().split(',')
            if len(parts) >= 4:  # Ensure that the line has at least 4 parts
                try:
                    ids.append(int(parts[0]))  # First part is the ID
                    times.append(float(parts[3]))  # Fourth part is the time
                except ValueError:
                    continue  # Skip lines with invalid data
    return ids, times

# Path to your log file
file_path = 'log.csv'  # Replace with your log file's path

# Extract data
ids, times = extract_data_from_log(file_path)

# Plotting the data
plt.figure(figsize=(10, 6))
plt.plot(ids, times, marker='o', linestyle='-', color='b')
plt.title('ID vs Time')
plt.xlabel('ID')
plt.ylabel('Time')
plt.grid(True)
plt.show()
