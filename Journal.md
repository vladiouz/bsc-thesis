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

