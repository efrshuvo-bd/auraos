# AuraOS OTA skeleton

This directory defines the **update contract** for a 4-year support window.

- `channels.json` — `os` / `agent` / `models` streams  
- `slots.json` — A/B slot state + rollback  
- `dev-keys/` — placeholder for development signing material (do not ship production secrets)  
- `apply_update.md` — operator notes for applying a slot update  

Production devices must replace dev keys with HSM-backed keys and enforce verified boot.
