Kin Backup
==========

Secure, simple backups for your [next of kin](https://en.wikipedia.org/wiki/Next_of_kin).

This project serves two main goals:

1. Make it easier to create and maintain secure backups of sensitive information that my non-technical close relatives can restore after I am hit by the proverbial Bus.
2. Help me learn Rust.

Overview
--------

Kin is a simple system that creates backup packages for your relatives to keep locked away in fire safes, safe deposit boxes, etc. It should definitely _not_ be used as an online backup, is definitely _not_ meant to be given to untrustworthy people, and will definitely _not_ protect you from the NSA.

Each backup package contains an unencrypted folder of public documents, for things like [last will and testament](https://en.wikipedia.org/wiki/Will_and_testament) documents or [living trust](https://en.wikipedia.org/wiki/Trust_law) documents. It will also contain an encrypted folder for things like password manager backups, two-factor authentication recovery codes, sensitive personal information, etc.

Of course giving any one person such sensitive "keys to the kingdom" is arguably a bad idea. Which is why Kin prevents any one individual from being able to decrypt the backup that they have access to. Kin is designed so that you give backups to at least 2 people. Each person cannot decrypt their own backup, but they can decrypt a backup that someone else holds. This isn't just to protect you from one family member going rogue, but also protects you when your trusted backup-holders experience burglary or otherwise lose track of your backups.

Kin also helps generate a clear, concise, non-technical readme for your relatives. It will tell them what they need to know to decrypt the backup (i.e. "You need a password from Bob or Sarah to get access to the sensitive information on this flash drive"). And it provides an executable for Windows, Mac OS, or Linux that can simply be double-clicked and walk the user through decrypting the backup.
