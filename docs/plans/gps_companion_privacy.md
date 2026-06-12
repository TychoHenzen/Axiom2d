# GPS Companion — Privacy Policy & Play Data-Safety Notes

> Draft stub. The owner must host the privacy policy at a public HTTPS URL and paste the
> data-safety answers into the Play Console. This app is designed to make those answers
> trivial: **it collects location only on-device and sends nothing to any server.**

## Privacy Policy (host this text publicly)

**GPS Companion — Privacy Policy**

GPS Companion uses your device's location to place collectible "grains" on a map near you.

- **Location data** is read on your device while the app is open and is used only to show
  your position and nearby grains. It is **not** transmitted to us or any third party and
  is **not** stored off-device.
- **Background location** (opt-in): When you enable "Track in Background" on the Map tab,
  the app uses an Android foreground service with a persistent notification to continue
  collecting location data while the app is minimized. Location data stays on-device and
  is never transmitted.
- **Collected grains and booster packs** are stored locally on your device. They are sent
  to another device **only** when you explicitly initiate a local-network (Wi-Fi) transfer
  by scanning a QR code shown on that device. No data leaves your local network.
- **Map tiles** are fetched from OpenStreetMap (`tile.openstreetmap.org`) to draw the
  map. These requests include your approximate map viewport (standard for any map app)
  but no account or identifier. To make the app fully offline, bundle local map tiles
  instead — see the build plan.
- **Biome data** (land-use labels) is fetched from the Overpass API
  (`overpass-api.de`). Overpass requests include the approximate map viewport bbox;
  no account or personal identifier is sent. Responses are cached on-device after the
  first fetch for a given area, so subsequent visits to the same region do not generate
  network traffic.
- **No accounts, no analytics, no advertising, no internet backend.**
- **Camera** is used only to scan the transfer QR code; no images are stored or sent.

Contact: <owner email>. Last updated: <date>.

## Play Console Data-Safety answers

| Question | Answer |
|----------|--------|
| Does your app collect/share user data? | Collects location; does **not** share it off-device |
| Location — precise | Yes, used app-functionality, **not** shared, **not** stored off-device |
| Data encrypted in transit | N/A (no server); LAN transfer is local-only, user-initiated |
| Can users request deletion | Yes — uninstall removes all local data |
| Data collected ephemerally | Location processed in memory for display |

## Android permissions (manifest) and rationale

| Permission | Why | Notes |
|-----------|-----|-------|
| `ACCESS_FINE_LOCATION` | Place GPS dot + nearby grains | Request at runtime |
| `ACCESS_COARSE_LOCATION` | Fallback / required pair with fine | Request at runtime |
| `CAMERA` | Scan transfer QR code | Request at runtime, only on Transfer screen |
| `INTERNET` | LAN WebSocket to paired desktop | Local network only |
| `ACCESS_NETWORK_STATE` | Detect Wi-Fi for LAN transfer | — |
| `FOREGROUND_SERVICE` | Opt-in background GPS tracking | User-initiated via toggle |
| `FOREGROUND_SERVICE_LOCATION` | Location type for foreground service | User-initiated via toggle |

Background location uses a foreground service with persistent notification (opt-in).
No `ACCESS_BACKGROUND_LOCATION` — location tracking stops if the user removes the
app from recents, keeping the data-safety profile and review simpler.
</content>
