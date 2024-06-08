# Solana Vested Funds
Supports SPL tokens, so you can vest custom tokens

## Prerequisite
- Node
- Solana wallet

## Usage
1. git clone `https://github.com/Spxc/solana-vesting-contract`
2. copy `example.env` to `.env`
3. `npm install`
4. `npm run`

## Testing
1. Ensure your TypeScript files are compiled to JavaScript using `tsc`
2. Use `mocha` to run the compiled JavaScript test file.
```bash
npx mocha dist/client.test.js --timeout 10000
```

## Todo
- [x] Write contract
- [x] Write nodejs client
- [x] Write nodejs tests
- [ ] Add support for env variables
- [ ] Write GHA CD/CI

## Contribution
.

## License
.
