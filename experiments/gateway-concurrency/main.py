import asyncio
import aiohttp
import time
import json
from typing import List, Tuple

class ConcurrencyTester:
    def __init__(self, base_url: str = "http://localhost:8079"):
        self.base_url = base_url
        self.test_payload = {
            "scAddress": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "funcName": "getTotalFeePercent"
        }
    
    async def make_request(self, session: aiohttp.ClientSession, request_id: int) -> Tuple[int, bool, float, str]:
        """Make a single request and return (id, success, response_time, error_msg)"""
        start_time = time.time()
        try:
            async with session.post(
                f"{self.base_url}/vm-values/int",
                json=self.test_payload,
                timeout=aiohttp.ClientTimeout(total=30)
            ) as response:
                await response.text()
                response_time = time.time() - start_time
                return (request_id, response.status == 200, response_time, "")
        except Exception as e:
            response_time = time.time() - start_time
            return (request_id, False, response_time, str(e))
    
    async def test_concurrent_requests(self, num_requests: int) -> dict:
        """Test a specific number of concurrent requests"""
        print(f"Testing {num_requests} concurrent requests...")
        
        connector = aiohttp.TCPConnector(limit=num_requests + 10)
        timeout = aiohttp.ClientTimeout(total=60)
        
        async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
            start_time = time.time()
            
            # Create all requests
            tasks = [
                self.make_request(session, i) 
                for i in range(num_requests)
            ]
            
            # Execute all requests concurrently
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            total_time = time.time() - start_time
            
            # Process results
            successful = 0
            failed = 0
            response_times = []
            errors = []
            
            for result in results:
                if isinstance(result, Exception):
                    failed += 1
                    errors.append(str(result))
                else:
                    req_id, success, resp_time, error_msg = result
                    if success:
                        successful += 1
                        response_times.append(resp_time)
                    else:
                        failed += 1
                        errors.append(error_msg or "HTTP error")
            
            return {
                "num_requests": num_requests,
                "successful": successful,
                "failed": failed,
                "success_rate": successful / num_requests * 100,
                "total_time": total_time,
                "avg_response_time": sum(response_times) / len(response_times) if response_times else 0,
                "max_response_time": max(response_times) if response_times else 0,
                "min_response_time": min(response_times) if response_times else 0,
                "errors": list(set(errors))[:5]  # Show unique errors, max 5
            }
    
    async def find_max_concurrent_requests(self, start: int = 10, max_test: int = 1000, step: int = 50):
        """Binary search to find maximum concurrent requests with >95% success rate"""
        print("Finding maximum concurrent requests...")
        print("=" * 60)
        
        results = []
        
        # Test increasing numbers of concurrent requests
        for num_requests in range(start, max_test + 1, step):
            result = await self.test_concurrent_requests(num_requests)
            results.append(result)
            
            print(f"Requests: {num_requests:4d} | "
                  f"Success: {result['successful']:4d}/{num_requests:4d} "
                  f"({result['success_rate']:5.1f}%) | "
                  f"Total Time: {result['total_time']:6.2f}s | "
                  f"Avg Response: {result['avg_response_time']:6.3f}s")
            
            # If success rate drops below 95%, we've likely hit the limit
            if result['success_rate'] < 95:
                print(f"\nSuccess rate dropped below 95% at {num_requests} requests")
                break
            
            # Add a small delay between tests
            await asyncio.sleep(2)
        
        return results
    
    def print_detailed_results(self, results: List[dict]):
        """Print detailed analysis of results"""
        print("\n" + "=" * 80)
        print("DETAILED RESULTS")
        print("=" * 80)
        
        for result in results:
            print(f"\n{result['num_requests']} Concurrent Requests:")
            print(f"  Success Rate: {result['success_rate']:.1f}% ({result['successful']}/{result['num_requests']})")
            print(f"  Total Time: {result['total_time']:.2f}s")
            print(f"  Response Times: avg={result['avg_response_time']:.3f}s, "
                  f"min={result['min_response_time']:.3f}s, max={result['max_response_time']:.3f}s")
            if result['errors']:
                print(f"  Errors: {result['errors']}")
        
        # Find the sweet spot
        good_results = [r for r in results if r['success_rate'] >= 95]
        if good_results:
            best = max(good_results, key=lambda x: x['num_requests'])
            print(f"\n🎯 RECOMMENDATION:")
            print(f"   Maximum safe concurrent requests: {best['num_requests']}")
            print(f"   Success rate: {best['success_rate']:.1f}%")
            print(f"   Average response time: {best['avg_response_time']:.3f}s")

async def main():
    # Test your local observer/proxy
    tester = ConcurrencyTester("http://localhost:8079")
    
    print("MultiversX Observer/Proxy Concurrency Test")
    print("=" * 50)
    print("Testing endpoint: /vm-values/int")
    print("Target: http://localhost:8079")
    print()
    
    # Run the test
    results = await tester.find_max_concurrent_requests(
        start=50,      # Start with 100 concurrent requests
        max_test=80,  # Test up to 800 (since you mentioned ~750)
        step=1        # Increase by 25 each time
    )
    
    # Print detailed analysis
    tester.print_detailed_results(results)
    
    # Save results to file
    with open("concurrency_test_results.json", "w") as f:
        json.dump(results, f, indent=2)
    print(f"\n📁 Results saved to: concurrency_test_results.json")

if __name__ == "__main__":
    asyncio.run(main())
