<div align="center">
<img alt="Logo Icon" height="256" src="assets/icon.svg" />

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![licence badge](https://img.shields.io/github/license/mgziminsky/allaerris)](LICENSE.txt)
[![ci.yml](https://github.com/mgziminsky/allaerris/actions/workflows/ci.yml/badge.svg)](.github/workflows/ci.yml)

<div align="initial">

Another Minecraft mod manager, heavily inspired by [ferium].
The project names are a [portmanteau] of [ferris] and [allay].
In my head lore, the core lib [`ferrallay`] can be thought of as a "species" of [allay] with claws for arms, and the cli [`allaerris`] is the name of one such individual of said species.

This project was initially going to be some feature PRs to the [libium] and [ferium] projects by [gorilla-devs],
but it quickly became apparent the features and functionality I wanted didn't fit into the existing architecture.
Since I needed a project to keep me busy and practice with, I decided to basically start over and build what I wanted.

I mainly wanted something that provided more control over profile mods, especially when using a modpack. The primary features/differences from [ferium] are:

- Allow overriding and/or excluding mods from a modpack
- Support locking mods to a specific version
- Selectively update installed mods/packs
- Keep track of installed files for cleaner updates
- Don't touch files from other sources to support easy manual customization
- Detect profile of working directory tree and use it when present
- Globally cached file downloads
- API clients generated from OpenAPI specs to support full functionality
- Support for installing server launchers

![Static Badge](https://img.shields.io/badge/XMR-gray?logo=monero)
`84s5qn5gxEAM3mNSGjsb6k5xTDpsFuruRgt6yVkbyDzuc56nyuga3mrdnZoq3vWUyi1fRLYciTNiPUL4zvBrESgYEhCZWzA`


[gorilla-devs]: https://github.com/gorilla-devs
[libium]: https://github.com/gorilla-devs/libium
[ferium]: https://github.com/gorilla-devs/ferium
[portmanteau]: https://farside.link/wikipedia.org/wiki/Blend_word
[ferris]: https://rustacean.net/
[allay]: https://minecraft.wiki/w/Allay
[`ferrallay`]: ./lib
[`allaerris`]: ./cli
