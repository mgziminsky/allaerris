# Ferrallay
This lib contains the core functionality of this project. It handles the bulk of the management functions for mods, config, and profiles,
as well as providing a normalized client interface for interacting with the different supported service APIs.

The primary modules are:
- [`client`](src/client.rs) - Public client interface wrapping the raw service APIs
- [`config`](src/config.rs) - Types dealing with the configuration data that is saved/loaded to/from the filesystem
- [`mgmt`](src/mgmt.rs) - Profile management features dealing with the actual download/install/update fuctionality
