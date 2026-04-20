import asyncio
import aiohttp
import json
from typing import Dict, List, Set, Tuple

class PoolFilter:
    def __init__(self, base_token_id: str = "USDC-350c4e"):
        self.base_token_id = base_token_id
        self.base_api = "https://devnet-api.multiversx.com"
    
    async def fetch_all_pools(self) -> List[Dict]:
        """Fetch all liquidity pools from the API"""
        print("Fetching all pools from API...")
        
        async with aiohttp.ClientSession() as session:
            url = f"{self.base_api}/mex/pairs?size=800&exchange=xexchange&includeFarms=false"
            
            try:
                async with session.get(url) as response:
                    if response.status == 200:
                        pools = await response.json()
                        print(f"Fetched {len(pools)} total pools")
                        return pools
                    else:
                        print(f"API request failed with status: {response.status}")
                        return []
            except Exception as e:
                print(f"Error fetching pools: {e}")
                return []
    
    def filter_pools_for_arbitrage(self, all_pools: List[Dict]) -> List[Dict]:
        """
        Filter pools for 3-step arbitrage: BASE -> Token A -> Token B -> BASE
        Only include pools that can participate in triangular arbitrage
        """
        print(f"\nFiltering pools for 3-step arbitrage with BASE_TOKEN_ID: {self.base_token_id}")
        
        # Build adjacency graph of all token pairs
        token_graph = {}
        pool_map = {}  # Maps (token1, token2) -> pool
        
        for pool in all_pools:
            base_id = pool.get("baseId", "")
            quote_id = pool.get("quoteId", "")
            
            if not base_id or not quote_id:
                continue
                
            # Add bidirectional edges
            if base_id not in token_graph:
                token_graph[base_id] = set()
            if quote_id not in token_graph:
                token_graph[quote_id] = set()
                
            token_graph[base_id].add(quote_id)
            token_graph[quote_id].add(base_id)
            
            # Store pool for both directions
            pool_map[(base_id, quote_id)] = pool
            pool_map[(quote_id, base_id)] = pool
        
        print(f"Built token graph with {len(token_graph)} tokens")
        
        # Find all possible triangular arbitrage paths: BASE -> A -> B -> BASE
        arbitrage_pools = {}  # Use dict with pool address as key to avoid duplicates
        triangular_paths = []
        
        if self.base_token_id not in token_graph:
            print(f"BASE_TOKEN_ID {self.base_token_id} not found in any pools!")
            return []
        
        base_neighbors = token_graph[self.base_token_id]
        print(f"BASE token {self.base_token_id} connects to {len(base_neighbors)} tokens")
        
        # For each token A connected to BASE
        for token_a in base_neighbors:
            if token_a == self.base_token_id:
                continue
                
            # For each token B connected to A (but not BASE)
            if token_a in token_graph:
                for token_b in token_graph[token_a]:
                    if token_b == self.base_token_id or token_b == token_a:
                        continue
                    
                    # Check if B connects back to BASE (completing the triangle)
                    if token_b in token_graph and self.base_token_id in token_graph[token_b]:
                        # Found a triangular path: BASE -> A -> B -> BASE
                        path = (self.base_token_id, token_a, token_b)
                        triangular_paths.append(path)
                        
                        # Add all three pools to our arbitrage dict (using address as key)
                        pool1 = pool_map[(self.base_token_id, token_a)]
                        pool2 = pool_map[(token_a, token_b)]
                        pool3 = pool_map[(token_b, self.base_token_id)]
                        
                        arbitrage_pools[pool1.get('address', '')] = pool1
                        arbitrage_pools[pool2.get('address', '')] = pool2
                        arbitrage_pools[pool3.get('address', '')] = pool3
        
        arbitrage_pools_list = list(arbitrage_pools.values())
        
        print(f"Found {len(triangular_paths)} triangular arbitrage paths")
        print(f"Total pools needed for arbitrage: {len(arbitrage_pools_list)}")
        
        # Show some example paths
        if triangular_paths:
            print(f"\nExample arbitrage paths:")
            for i, (base, token_a, token_b) in enumerate(triangular_paths[:10]):
                print(f"  {i+1}. {base} -> {token_a} -> {token_b} -> {base}")
            if len(triangular_paths) > 10:
                print(f"  ... and {len(triangular_paths) - 10} more paths")
        
        return arbitrage_pools_list
    
    def save_arbitrage_pools(self, arbitrage_pools: List[Dict], output_dir: str = "arbitrage_pools"):
        """Save arbitrage pools to CSV and JSON files"""
        import os
        import csv
        
        # Create output directory
        os.makedirs(output_dir, exist_ok=True)
        
        # Save arbitrage pools CSV
        csv_path = os.path.join(output_dir, "arbitrage_pools.csv")
        with open(csv_path, 'w', newline='') as csvfile:
            writer = csv.writer(csvfile)
            writer.writerow(['address', 'token1', 'token2'])
            for pool in arbitrage_pools:
                writer.writerow([
                    pool.get('address', ''),
                    pool.get('baseId', ''),
                    pool.get('quoteId', '')
                ])
        
        # Save arbitrage pools JSON
        json_path = os.path.join(output_dir, "arbitrage_pools.json")
        with open(json_path, 'w') as jsonfile:
            json.dump({
                'base_token_id': self.base_token_id,
                'arbitrage_pools': arbitrage_pools,
                'total_pools': len(arbitrage_pools),
                'description': 'Pools that can participate in 3-step triangular arbitrage'
            }, jsonfile, indent=2)
        
        print(f"\n📁 Files saved to '{output_dir}/' directory:")
        print(f"   - arbitrage_pools.csv ({len(arbitrage_pools)} pools)")
        print(f"   - arbitrage_pools.json (full pool data)")
        
        return len(arbitrage_pools)
    
    def print_arbitrage_analysis(self, arbitrage_pools: List[Dict]):
        """Print detailed analysis of arbitrage pools"""
        print(f"\n{'='*60}")
        print("TRIANGULAR ARBITRAGE POOL ANALYSIS")
        print(f"{'='*60}")
        print(f"Base Token ID: {self.base_token_id}")
        print(f"Arbitrage pools (can participate in 3-step arbitrage): {len(arbitrage_pools)}")
        
        # Categorize pools by their relationship to base token
        base_pools = []  # Pools containing base token
        intermediate_pools = []  # Pools between other tokens
        
        for pool in arbitrage_pools:
            base_id = pool.get('baseId', '')
            quote_id = pool.get('quoteId', '')
            
            if base_id == self.base_token_id or quote_id == self.base_token_id:
                base_pools.append(pool)
            else:
                intermediate_pools.append(pool)
        
        print(f"\nPool breakdown:")
        print(f"  - Pools with {self.base_token_id}: {len(base_pools)}")
        print(f"  - Intermediate pools: {len(intermediate_pools)}")
        
        # Show some examples
        if base_pools:
            print(f"\nBase token pool examples:")
            for i, pool in enumerate(base_pools[:5]):
                base_id = pool.get('baseId', '')
                quote_id = pool.get('quoteId', '')
                print(f"  {i+1}. {base_id} <-> {quote_id}")
            if len(base_pools) > 5:
                print(f"  ... and {len(base_pools) - 5} more")
        
        if intermediate_pools:
            print(f"\nIntermediate pool examples:")
            for i, pool in enumerate(intermediate_pools[:5]):
                base_id = pool.get('baseId', '')
                quote_id = pool.get('quoteId', '')
                print(f"  {i+1}. {base_id} <-> {quote_id}")
            if len(intermediate_pools) > 5:
                print(f"  ... and {len(intermediate_pools) - 5} more")

async def main():
    # Initialize the filter with your BASE_TOKEN_ID
    filter_tool = PoolFilter(base_token_id="USDC-350c4e")
    
    print("MultiversX Liquidity Pool Filter")
    print("=" * 40)
    
    # Fetch all pools
    all_pools = await filter_tool.fetch_all_pools()
    
    if not all_pools:
        print("Failed to fetch pools. Exiting.")
        return
    
    # Filter pools for triangular arbitrage
    arbitrage_pools = filter_tool.filter_pools_for_arbitrage(all_pools)
    
    # Print analysis
    filter_tool.print_arbitrage_analysis(arbitrage_pools)
    
    # Save arbitrage pools
    total_filtered = filter_tool.save_arbitrage_pools(arbitrage_pools)
    
    print(f"\n🎯 SUMMARY:")
    print(f"   Original pools: {len(all_pools)}")
    print(f"   Filtered pools: {total_filtered}")
    print(f"   Reduction: {((len(all_pools) - total_filtered) / len(all_pools) * 100):.1f}%")
    
    # Calculate potential request reduction
    original_requests = len(all_pools) * 3  # 3 requests per pool
    filtered_requests = total_filtered * 3
    print(f"\n📊 REQUEST IMPACT:")
    print(f"   Original requests per cycle: {original_requests}")
    print(f"   Filtered requests per cycle: {filtered_requests}")
    print(f"   Request reduction: {original_requests - filtered_requests} ({((original_requests - filtered_requests) / original_requests * 100):.1f}%)")

if __name__ == "__main__":
    asyncio.run(main())
