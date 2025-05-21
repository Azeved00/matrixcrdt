import csv
import pandas as pd

operation_map = {
    " get_pharmacy_prescriptions": ["stateful_query", "stateless_query"],
    " get_prescription_medication": ["stateful_query", "stateless_query"],
    " get_staff_prescriptions": ["stateful_query", "stateless_query"],
    " get_processed_pharmacy_prescriptions": ["stateful_query", "stateless_query"],
    " get_patient": ["stateful_query", "stateless_query"],
    " get_prescription": ["stateful_query", "stateless_query"],

    " create_prescription": ["apply"],
    " process_prescription": ["apply"],
    " update_prescription_medication": ["apply"],
}

def validate_df(df: pd.DataFrame, max_errors=10):
    errors = []

    if df.empty:
        errors.append("DataFrame is empty.")
        return errors

    expected_ids = {}
    expected_back_ids = {}

    for idx, row in df.iterrows():
        line_number = idx + 2  
        line_data = row.to_dict()

        # Rule 1: Operation map
        client_op = row["client_operation"]
        dag_op = row["dag_operation"]
        if client_op in operation_map and dag_op not in operation_map[client_op]:
            errors.append(
                f"Line {line_number}: Invalid dag_operation '{dag_op}' for client_operation '{client_op}' - \n{line_data}"
            )

        # Rule 2: total_time > front_time + back_time
        #try:
        #    total = float(row["total_time"])
        #    front = float(row["front_time"])
        #    back = float(row["back_time"])
        #    if total <= front + back:
        #        errors.append(
                #            f"Line {line_number}: total_time ({total}) is not greater than front_time + back_time ({front + back}) - \n{line_data}"
                #)
        #except ValueError:
        #    errors.append(f"Line {line_number}: Invalid numeric time values - \n{line_data}")


        try:
            thread_id = int(row["thread_id"])
            front_id = int(row["front_id"])
            back_id= int(row["back_id"])
        except ValueError:
            errors.append(f"Line {line_number}: Invalid numeric time values - \n{line_data}")


        # Rule 3: Sequential id within thread
        # Rule 4: Sequential back_id within thread
        if thread_id not in expected_ids:
            if front_id != -1:
                expected_ids[thread_id] = front_id
                expected_ids[thread_id] += 1
            if back_id != -1:
                expected_back_ids[thread_id] = back_id
                expected_back_ids[thread_id] += 1
        else :
            if front_id != expected_ids[thread_id] and front_id != -1:
                errors.append(
                    f"Line {line_number}: Expected id {expected_ids[thread_id]} for thread {thread_id}, found {front_id} - \n{line_data}"
                )

            if back_id != expected_back_ids[thread_id] and back_id != -1:
                errors.append(
                    f"Line {line_number}: Expected back_id {expected_back_ids[thread_id]} for thread {thread_id}, found {back_id} - \n{line_data}"
                )
            expected_ids[thread_id] += 1
            expected_back_ids[thread_id] += 1
    return errors

if __name__=="__main__":
    file_path = 'final.csv'

    df = pd.read_csv(file_path)
    validation_errors = validate_df(df)

    if validation_errors:
        for err in validation_errors:
            print(err)
    else:
        print("CSV passed all validation checks.")
