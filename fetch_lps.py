import requests

search_term = "liquidity pool"
page_size = 25
total_pages = 11

output_file = "contracts.txt"

contracts = []

for page in range(total_pages):
    offset = page * page_size
    url = f"https://devnet-api.multiversx.com/accounts?from={offset}&size={page_size}&isSmartContract=true&search=liquidity+pool"
    response = requests.get(url)
    response.raise_for_status()
    data = response.json()

    for app in data:
        address = app.get("address")
        if address:
            contracts.append(address)

with open(output_file, "w") as f:
    for address in contracts:
        f.write(address + "\n")
