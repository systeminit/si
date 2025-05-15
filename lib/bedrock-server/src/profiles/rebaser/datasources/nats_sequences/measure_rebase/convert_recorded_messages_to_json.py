import re
import json
import binascii

def hexify_cbor_line(line: str) -> str:
    return binascii.hexlify(line.encode("latin1")).decode("utf-8")

with open("recorded_messages.txt", encoding="latin1") as f:
    content = f.read()

entries = content.split("Acknowledged message")

messages = []

for entry in entries:
    if not entry.strip():
        continue

    # Extract subject
    subject_match = re.search(r"subj:\s+([^\s/]+)", entry)
    subject = subject_match.group(1) if subject_match else "UNKNOWN"

    # Extract headers
    headers = {}
    for line in entry.splitlines():
        header_match = re.match(r"\s{2}([A-Za-z0-9\-]+):\s(.*)", line)
        if header_match:
            key = header_match.group(1)
            value = header_match.group(2)
            headers[key] = value

    # Extract payload from Data block
    lines = entry.splitlines()
    payload_hex = ""
    in_data_section = False
    for line in lines:
        if line.strip() == "Data:":
            in_data_section = True
            continue
        if in_data_section:
            striped = line.strip()
            if striped:
                payload_hex = hexify_cbor_line(striped)
                break

    messages.append({
        "subject": subject,
        "headers": headers,
        "payload_hex": payload_hex
    })

# Write output
with open("sequence.json", "w") as f:
    json.dump(messages, f, indent=2)

print(f"✅ Extracted {len(messages)} messages to sequence.json")

