Kin Backup
==========

Secure, simple backups for your [next of kin](https://en.wikipedia.org/wiki/Next_of_kin).

This project serves two main goals:

1. Make it easier to create and maintain secure backups of sensitive information that my non-technical close relatives can restore after I am hit by the proverbial Bus.
2. Teach me Rust.

Overview
--------

**tl;dr:** Create several small encrypted backups and distribute them to a few trusted family members or friends. No individual can decrypt their own copy of the backup, but they can decrypt someone else's. Thus when something happens to you, two trusted people together have what they need to gain access to your sensitive information.

**The detailed version:**

Kin is a simple system that creates backup packages for your relatives to keep locked away in fire safes, safe deposit boxes, etc. Each backup package contains an unencrypted zip archive of public documents, for things like [last will and testament](https://en.wikipedia.org/wiki/Will_and_testament) documents or [living trust](https://en.wikipedia.org/wiki/Trust_law) documents. It also contains an encrypted archive for things like password manager backups, two-factor authentication recovery codes, sensitive personal information, etc.

Of course giving any one person such sensitive "keys to the kingdom" is arguably a bad idea. Which is why Kin prevents any one individual from being able to decrypt the backup that they have access to. Kin is designed so that you give backups to at least two different people (recommended three or more). Each person cannot decrypt their own backup, but they can decrypt a backup that someone else holds. This isn't just to protect you from one family member going rogue, but also protects you when your trusted backup-holders experience burglary or otherwise lose track of your backups.

Kin also helps generate a clear, concise, non-technical readme HTML file for your relatives. You can customize the readme however you want using a markdown template. In version 1.0, this readme file will have a small self-contained Javascript app embedded in it which will decrypt the backup. For now, a Linux-only `decrypt` executable comes with each backup package.

Encryption is implemented using the widely-used and trusted [libsodium](https://download.libsodium.org/doc/) library. That said, Kin should definitely _not_ be used as an online backup, is definitely _not_ meant to be given to untrustworthy people, and will definitely _not_ protect you from the NSA.

Build Requirements
------------------

Kin Backup is created using Rust, so you need a Rust development environment. Since we're using libsodium, to build this project you'll also need a C compiler (`cc`, `clang`, etc) and `libssl-dev` (assuming you're on Ubuntu).

Getting Started
---------------

Once you have Kin installed, it's time to create an empty backup project. **Recommended:** Because you will probably be storing sensitive files in this backup project, you should create it in a location where drive encryption is enabled.

Assuming your name is Owen, and you want to give backups to three trusted family members named Alice, Bob, and Chuck:

```bash
mkdir backup_project
cd backup_project
kin init --owner Owen --recipients Alice Bob Chuck
```

This creates three things:

* A `public` folder where you put your public files. These will _not_ be encrypted.
* A `private` folder where you put all your private files. These will be encrypted.
* A `readme-template.md` file, which is a [markdown](https://github.com/adam-p/markdown-here/wiki/Markdown-Cheatsheet) file that will be used to generate nice looking readme files for your backup recipients.

Once you put the appropriate files in the `public` and `private` folders, and make adjustments to `readme-template.md` so it looks the way you want, it's time to compile backup packages for Alice, Bob, and Chuck.

Insert a USB flash drive into your computer and mount it. Assuming your flash drive is mounted at `/media/flash_drive/`:

```bash
kin compile --recipient Alice /media/flash_drive/
```

Feel free to inspect the contents of the flash drive now. You'll notice:

* A `public.zip` file that contains all the files in your `public` folder
* A `private.kin` file, which is an encrypted archive that contains all the files in your `private` folder
* A `readme.html` file, which when opened, explains what this is and how to decrypt the backup. Notice that it shows you Alice's randomly-generated passphrase.
* A `decrypt` program, which if you run it, will decrypt the `private.kin` file when you enter either Bob's or Chuck's passphrase. If you try to enter Alice's passphrase, the decryption will fail; you cannot decrypt Alice's backup with Alice's passphrase.

Now eject the flash drive and insert a new one for Bob. Run the same `compile` command as above, except with "Bob" as the recipient. Now run the `decrypt` program on Bob's flash drive, but use _Alice's_ passphrase. The decryption will succeed.

Create a backup package for Chuck as well, and then distribute your three flash drives to your three recipients. Now when you get hit by a bus, they won't be up a creek when they need to access your super secret important stuff.
