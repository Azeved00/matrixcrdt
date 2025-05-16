import csv
import pandas as pd

operation_map = {
    " get_pharmacy_prescriptions": ["stateful_query", "stateles_query"],
    " get_prescription_medication": ["stateful_query", "stateles_query"],
    " get_staff_prescriptions": ["stateful_query", "stateles_query"],
    " get_processed_pharmacy_prescriptions": ["stateful_query", "stateles_query"],
    " get_patient": ["stateful_query", "stateles_query"],
    " get_prescription": ["stateful_query", "stateles_query"],

    " create_prescription": ["apply"],
    " process_prescription": ["apply"],
    " update_prescription_medication": ["apply"],
}

def validate_df_single(df: pd.DataFrame, max_errors=10):
    errors = []

    if df.empty:
        errors.append("DataFrame is empty.")
        return errors

    try:
        expected_id = int(df.iloc[0]["id"])
        expected_back_id = int(df.iloc[0]["back_id"])
    except ValueError:
        errors.append("First row id or back_id is not an integer.")
        return errors

    for idx, row in df.iterrows():
        line_number = idx + 2  # Account for header + 0-based index

        try:
            current_id = int(row["id"])
            current_back_id = int(row["back_id"])
        except ValueError:
            errors.append(f"Line {line_number}: id or back_id is not an integer")
            if len(errors) >= max_errors:
                break
            continue

        # Rule 1: client_operation must match dag_operation
        client_op = row["client_operation"]
        dag_op = row["dag_operation"]
        if client_op in operation_map and dag_op not in operation_map[client_op]:
            errors.append(f"Line {line_number}: Invalid dag_operation '{dag_op}' for client_operation '{client_op}'")

        # Rule 2: total_time must be > front_time + back_time
        try:
            total = float(row["total_time"])
            front = float(row["front_time"])
            back = float(row["back_time"])
            if total <= front + back:
                errors.append(f"Line {line_number}: total_time ({total}) is not greater than front_time + back_time ({front + back})")
        except ValueError:
            errors.append(f"Line {line_number}: Invalid numeric time values")

        # Rule 3: id must be sequential starting from initial
        if current_id != expected_id:
            errors.append(f"Line {line_number}: Expected id {expected_id}, found {current_id}")
        expected_id += 1

        # Rule 4: back_id must be sequential starting from initial
        if current_back_id != expected_back_id:
            errors.append(f"Line {line_number}: Expected back_id {expected_back_id}, found {current_back_id}")
        expected_back_id += 1

    return errors



def validate_df(df: pd.DataFrame, max_errors=10):
    errors = []

    if df.empty:
        errors.append("DataFrame is empty.")
        return errors

    # Initialize expected ID/back_id counters per thread
    expected_ids = {}
    expected_back_ids = {}

    for idx, row in df.iterrows():
        line_number = idx + 2  # Account for header
        line_data = row.to_dict()

        try:
            thread_id = row["thread_id"]
            current_id = int(row["id"])
            current_back_id = int(row["back_id"])
        except ValueError:
            errors.append(
                f"Line {line_number}: Invalid thread_id, id, or back_id - {line_data}"
            )
            if len(errors) >= max_errors:
                break
            continue

        if thread_id not in expected_ids:
            expected_ids[thread_id] = current_id
            expected_back_ids[thread_id] = current_back_id

        # Rule 3: Sequential id within thread
        if current_id != expected_ids[thread_id]:
            errors.append(
                f"Line {line_number}: Expected id {expected_ids[thread_id]} for thread {thread_id}, found {current_id} - {line_data}"
            )

        # Rule 4: Sequential back_id within thread
        if current_back_id != expected_back_ids[thread_id]:
            errors.append(
                f"Line {line_number}: Expected back_id {expected_back_ids[thread_id]} for thread {thread_id}, found {current_back_id} - {line_data}"
            )

        expected_ids[thread_id] += 1
        expected_back_ids[thread_id] += 1

        # Rule 1: Operation map
        client_op = row["client_operation"]
        dag_op = row["dag_operation"]
        if client_op in operation_map and dag_op not in operation_map[client_op]:
            errors.append(
                f"Line {line_number}: Invalid dag_operation '{dag_op}' for client_operation '{client_op}' - {line_data}"
            )

        # Rule 2: total_time > front_time + back_time
        try:
            total = float(row["total_time"])
            front = float(row["front_time"])
            back = float(row["back_time"])
            if total <= front + back:
                errors.append(
                    f"Line {line_number}: total_time ({total}) is not greater than front_time + back_time ({front + back}) - {line_data}"
                )
        except ValueError:
            errors.append(f"Line {line_number}: Invalid numeric time values - {line_data}")

        if len(errors) >= max_errors:
            break

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
