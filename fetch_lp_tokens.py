import requests
import base64

API_URL = "https://devnet-gateway.multiversx.com/vm-values/string"


def get_token_ids(address):
    payload = {
        "scAddress": address,
        "funcName": "getFirstTokenId"
    }
    
    token_ids = []

    try:
        r = requests.post(API_URL, json=payload, timeout=10)
        r.raise_for_status()
        data = r.json()
        data = data.get("data", "")
        token_ids.append(data.get("data", ""))
    except Exception as e:
        return f"ERROR: {e}"
    
    payload["funcName"] = "getSecondTokenId"
    
    try:
        r = requests.post(API_URL, json=payload, timeout=10)
        r.raise_for_status()
        data = r.json()
        data = data.get("data", "")
        token_ids.append(data.get("data", ""))
    except Exception as e:
        return f"ERROR: {e}"
    
    return token_ids


def main():
    with open("contracts.txt", "r") as f, open("lp_tokens.txt", "w") as out:
        addresses = [line.strip() for line in f if line.strip()]

        for addr in addresses:
            result = get_token_ids(addr)
            out.write(f"{addr}: {result}\n")

if __name__ == "__main__":
    main()