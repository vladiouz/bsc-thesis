import csv
import ast

INPUT_FILE = "lp_tokens.txt"
OUTPUT_FILE = "pairs.csv"

with open(INPUT_FILE, "r") as infile, open(OUTPUT_FILE, "w", newline="") as outfile:
    writer = csv.writer(outfile)
    writer.writerow(["address", "token1", "token2"])

    for line in infile:
        if not line.strip():
            continue

        addr, tokens_str = line.split(":", 1)
        tokens = ast.literal_eval(tokens_str.strip())
        writer.writerow([addr.strip(), tokens[0], tokens[1]])