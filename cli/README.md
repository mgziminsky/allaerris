# Allaerris
Allaerris is a fast and feature rich CLI program for downloading and updating Minecraft mods from [Modrinth](https://modrinth.com/mods), [CurseForge](https://curseforge.com/minecraft/mc-mods), and [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases), and modpacks from [Modrinth](https://modrinth.com/modpacks) and [CurseForge](https://curseforge.com/minecraft/modpacks).
Simply specify the mods you use, and in just one command you can download the latest compatible version of all the mods you configured.

## Installation
### GitHub Releases
[![GitHub Releases](https://img.shields.io/github/v/release/mgziminsky/allaerris?color=bright-green&label=github%20releases)](https://github.com/mgziminsky/allaerris/releases)
> [!NOTE]
> You will have to manually download and install every time there is a new update.

1. Download the asset suitable for your operating system from the [latest release](https://github.com/mgziminsky/allaerris/releases/latest)
2. Unzip the file and move it to a folder in your path, e.g. `~/bin`
3. Remember to check the releases page for any updates!

## Overview / Help Page

> [!NOTE]
> A lot of allaerris' backend is in a separate project; [ferrallay](../lib).
> It contains most of the functional implementation of the project.

## Program Configuration

Allaerris stores profile and modpack information in its config file. By default, this is located at `~/.config/ferrallay/config.json`.
You can change this in 2 ways, setting the `ALLAERRIS_CONFIG_FILE` environment variable, or passing the `--config-file` global flag.
The flag always takes precedence.

You can also set a custom CurseForge API key or GitHub personal access token using the `CURSEFORGE_API_KEY` and `GITHUB_TOKEN` environment variables, or the `--curseforge_api_key` and `--github-token` global flags respectively.
Again, the flags take precedence.

## Quickstart

- [Create a new profile](#create) by running `allaerris profile new` and entering the details for your profile.
- [Add your mods](#adding) using `allaerris add`.
- [Set a modpack](#setting-a-modpack) using `allaerris modpack set`.
- Finally, download your mods using `allaerris apply`.

## Profiles
### Create
Create a profile by running `allaerris profile new` and specifying the following:
> [!TIP]
> You can also provide these settings as command line options to avoid interactivity for things like scripts
- Profile path
- Profile Name
- Minecraft version
- Mod loader

After creation, the new profile will automatically be set as the active profile.

### Edit
You can change the same settings afterwards by running `allaerris profile edit`. Again, you can provide these settings as options.

### Manage
You can get information about the current profile by running `allaerris profile` or `allaerris profile info`, and about all the profiles you have by running `allaerris profiles` or `allaerris profile list`.

Switch to a different profile using `allaerris profile switch`.

Delete a profile using `allaerris profile delete` and selecting the profile you want to delete or providing the profile name or path directly.


## Mods
### Adding
```bash
allaerris add <project_id>
```

#### Modrinth
`project_id` is the slug or project ID of the mod. (e.g. [Sodium](https://modrinth.com/mod/sodium) has the slug `sodium` and project ID `AANobbMI`). You can find the slug in the website URL (`modrinth.com/mod/<slug>`), and the project ID using the "Copy ID" button nested in the 3-dot menu.
So to add [Sodium](https://modrinth.com/mod/sodium), you can run `allaerris add sodium` or `allaerris add AANobbMI`.

#### CurseForge
`project_id` is the project ID of the mod. (e.g. [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith) has the project id `513688`). You can find the project id at the top of the sidebar under 'About Project'.
So to add [Terralith](https://www.curseforge.com/minecraft/mc-mods/terralith), you should run `allaerris add 513688`.

#### GitHub
`project_id` in the format `owner/repo` where `owner` is the username of the owner of the repository and `repo` is the name of the repository, both are case-insensitive. (e.g. [Sodium's repository](https://github.com/CaffeineMC/sodium-fabric) has the id `CaffeineMC/sodium-fabric`). You can find these at the top left of the repository's page as a big 'owner / name'.
So to add [Sodium](https://github.com/CaffeineMC/sodium-fabric), you should run `allaerris add CaffeineMC/sodium-fabric` (again, case-insensitive).
> [!IMPORTANT]
> The GitHub repository needs to upload built jar/zip/mrpack files to their Releases for allaerris to download, otherwise it will fail when applying the profile.

### Setting a Modpack
```bash
allaerris modpack set <project_id>
```
`profile_id` follows the same guidelines as for [adding mods](#adding-mods)

### Manage Mods
Download and install all mods to the active provile using `allaerris apply`.
If allaerris fails to download a mod, it will print its name and try to give a reason while continuing to download the rest of your mods.

You can list out all the mods in your current profile by running `allaerris list`. If you want to see more information about them, you can use `allaerris list -v`.

You can remove any of your mods using `allaerris remove`; just select the ones you would like to remove using the space key, and press enter once you're done. You can also provide the names or IDs of the mods to remove as arguments.

> [!NOTE]
> Both mod names and GitHub repository identifiers are case insensitive.
> Mod names with spaces have to be given in quotes (`allaerris remove "ok zoomer"`) or the spaces should be escaped (usually `allaerris remove ok\ zoomer`, but depends on the shell).

#### Updating
```bash
allaerris update
```
This will find the latest compatible version of all installed mods and, by default, mark them to be installed the next time `allaerris apply` is called. You may also specify only specific mods to be updated. There are also flags to cancel a pending update, or to immediately install all found updates.

## Feature Requests

If you would like to make a feature request, check the [issue tracker](https://github.com/mgziminsky/allaerris/issues?q=is%3Aissue+label%3Aenhancement) to see if the feature has already been added or is planned.
If not, [create a new issue](https://github.com/mgziminsky/allaerris/issues/new/choose).
