# Paperspace: A literature exploration tool.

WORK IN PROGRESS

## Why?
Finding relevant papers is hard! Semantic Scholar has a good selection of tools to do this with, but their UI isn't the best for exploration of new fields. Paperspace hopes to provide a UI that can pick out relevant papers quickly and easily.

## Building - Devs only at the moment, sorry!
Make sure you have installed:

+ [Rust](https://www.rust-lang.org/)
+ [Node.js](https://nodejs.org/en/) - Windows users check [here](https://docs.microsoft.com/en-us/windows/dev-environment/javascript/nodejs-on-windows)

Clone repo and run in the root directory:
```shell
cargo run
```

This should automatically:
+ Pull in the npm dependencies,
+ Build the frontend
+ Pull in and build Rust dependencies
+ Build executable and embed frontend 
+ Run a debug version that will open in the browser.

## Development
### Backend 
Development of backend is as can be expected, edit code and see if it works etc.

### Frontend 
Thankfully you don't need to know Rust for this! Paperspace's frontend is a SvelteKit app that can be developed and iterated on as any other.

You can use the npm dev server to speed up iteration.
In one terminal run:
```shell
cargo run
```
This will launch a Paperspace instance on `localhost:4000`. We can't edit the frontend of this instance, but we can proxy requests to it's backend.
In another terminal, cd into the `frontend` directory and run:
```shell
npm run dev
```
The npm development web server will boot up and open a page at `localhost:XXXX`. The page will will automatically reload when frontend files are saved. API requests are proxied to the running Paperspace instance. 

Any changes made to frontend code should be picked up by `cargo run` and/or `cargo build`. This will automatically recomple the frontend for deployment and embed the updated version in the new executable.