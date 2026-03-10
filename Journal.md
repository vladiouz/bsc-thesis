## 17.02

- generated empty sc with sc-meta
- sc should have the following capabilities:
	- users can stake in the currency of the sc (I'm thinking of `USDC`, but will have a variable for it) => need unstake operation too (for the whole amount, part of it or just the winnings)
	- winnings are proportional to the amount staked => should keep track of staked amount and winnings for each user (note: could use `VecMapper` for staked and `SingleValueMapper` for winnings, maybe)
	- admin should have an endpoint to execute specific trades of the DEX
- I'm thinking that other components should be:
	- a script that fetches DEX values, plans out the trades and executes them on the sc (offchain)
	- frontend for users to easily interact with the sc

## 18.02
- we can use `SingleValueMapper` with `ManagedAddress` as key for user winnings, as there's no real need to loop through winnings and it's more efficient than other storage types
- for the staked amount, we need both access by key and loop through; `VecMapper` only supports looping, so we cancel it out; `MapMapper` supports both, even though it's more expensive
- as a side note, I can also create only one `MapMapper` to contain data for both staked and winnings and it might look cleaner, but my approach is more efficient
- added storage to the sc
- created repo
- added a few basic owner interactions to the sc

## 19.02
- `stake()`
- look into arbitrage
- https://uk.mathworks.com/discovery/statistical-arbitrage.html -> good for future studies, maybe
- https://www.litefinance.org/blog/for-beginners/arbitrage-trading/ -> not too relevant tbh
- decided to go along with the flow and just use the MvX API for now

## 23.02
- a bit of research on the information I need to run my arbitrage bot
- relevant data:
	- pool reserves
	- swap fee
	- slippage
	- gas costs
	- execution latency (not that important for devnet, but might be if going mainnet)
- [here](https://docs.multiversx.com/developers/tutorials/dex-walkthrough/#swap-tokens-fixed-input) we can see formulas for the fees: `rI∗rO=(rI+(1−f)∗aI)∗(rO−aO)`
- would be nice to look into `multi_pair_swap` of the Router SC (dex > router > src > multi_pair_swap.rs)

## 01.03
- found out how to get info about trades:
	- POST at `https://devnet-gateway.multiversx.com/vm-values/int` endpoint
	- add the SC address in the body and the function name
	- `getReserve` with a hex encoded token ID as argument for getting the token amount inside the LP
	- `getTotalFeePercent` for the swap fee (from what I saw, it's usually 0.3% or 1%)
	- example body: 
	```
	{
  		"scAddress": "erd1qqqqqqqqqqqqqpgqus9r9gwtg24a9fvzv743hgydecpkxs8q0n4szz2az0",
  		"funcName": "getReserve",
  		"args": ["5745474c442d613238633539"]
	}
	```
	- with this data and the swap formula, we can accurately calculate the output amount for a given input (tested on ITHEUM-WEGLD pair and value matches the one on xExchange)
- the system should work as such:
	- rarely, maybe once a day, a script will look out for new LPs
	- the main script will run a loop each second, will use the LP addresses from the other script and fetch reserves and fees, create the graph and execute the trades via the SC

## 02.03
- I am going to write the graph creation script
- retrieving reserves and fees for all LPs takes 20-25 seconds (probably because of throttling) - I searched and a solution could be to have an observer node, but that will be done later

## 03.03
- wrote `swap` and `simulate_triangle` functions, now looking for finding triangles
- just for optimization, nodes with degree 1 will be ignored (todo later)
- wrote a basic script for triangular arbitrage using USDC as the base currrency
- **to do a cleaner job in calling SCs from my SC, I can look at caller-sc project in my MvX folder**

## 05.03
- added the `execute_trades` function in the SC (in a a simpler state for now) to make sure that the SC can call the `swapTokensFixedInput` in the LPs and I tested it with the interactor
- next up: add SC logic to complete the triangle, call from bot, optimizations

## 06.03
- updated the SC function to receive more trades (basically like the Router SC) -> if it's not going to work, then I'll just call a method in the router SC
- tested the endpoint on chain and it worked brilliant, transaction is right [here](https://devnet-explorer.multiversx.com/transactions/d77f3d61946fcf0ce99455e95638e1abf286ccd89c8fabfe827fd142913e223d)
- next: call from bot, finish the SC, optimizations, frontend

## 09.03
- copied the interactor into the `arbitrage-bot` folder and called the `execute_trades` method from the script; currently there might be some issues with the MvX devnet API and no pairs are being retrieved
- continued work on SC, must check if `execute_trades` is done and them move on to the other functionalities

## 10.03
- for the last 2 days, the devnet api had some issues with the `/mex/pairs` endpoint so I wrote a python script that fetched me the LPs just in case the api issue won't get solved; it's output is in `contracts.txt`
- completed (hopefully) the SC with stake, unstake, restake and claim winnings functions
- next: clean up off-chain code
